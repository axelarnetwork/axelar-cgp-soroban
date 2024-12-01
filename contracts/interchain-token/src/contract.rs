use soroban_sdk::token::{self, Interface as _};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use crate::utils::{
    check_nonnegative_amount, extend_instance_ttl, read_allowance, read_balance, read_decimal,
    read_name, read_symbol, receive_balance, spend_allowance, spend_balance, write_allowance,
    write_metadata,
};
use axelar_soroban_std::shared_interfaces::{migrate, UpgradableInterface};
use axelar_soroban_std::shared_interfaces::{MigratableInterface, OwnableInterface};
use axelar_soroban_std::{ensure, shared_interfaces};

use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, IntoVal, String};

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(
        env: Env,
        owner: Address,
        minter: Address,
        interchain_token_service: Address,
        token_id: Bytes,
        token_meta_data: TokenMetadata,
    ) -> Result<(), ContractError> {
        shared_interfaces::set_owner(&env, &owner);

        ensure!(!token_id.is_empty(), ContractError::TokenIdZero);

        Self::validate_token_metadata(token_meta_data.clone())?;

        env.storage().instance().set(&DataKey::TokenId, &token_id);

        write_metadata(&env, token_meta_data);

        env.storage()
            .persistent()
            .set(&DataKey::Minter(minter), &true);
        env.storage()
            .persistent()
            .set(&DataKey::Minter(interchain_token_service.clone()), &true);
        env.storage()
            .instance()
            .set(&DataKey::InterchainTokenService, &interchain_token_service);

        Ok(())
    }

    pub fn validate_token_metadata(token_meta_data: TokenMetadata) -> Result<(), ContractError> {
        ensure!(
            token_meta_data.decimal <= u8::MAX.into(),
            ContractError::InvalidDecimal
        );
        ensure!(
            !token_meta_data.name.is_empty(),
            ContractError::TokenNameEmpty
        );
        ensure!(
            !token_meta_data.symbol.is_empty(),
            ContractError::TokenSymbolEmpty
        );
        Ok(())
    }

    pub fn token_id(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::TokenId)
            .expect("token id not found")
    }

    pub fn interchain_token_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenService)
            .expect("interchain token service not found")
    }

    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        minter.require_auth_for_args((&to, amount).into_val(&env));

        check_nonnegative_amount(amount);

        let is_authorized = env
            .storage()
            .persistent()
            .get::<_, bool>(&DataKey::Minter(minter.clone()))
            .unwrap_or(false);

        if !is_authorized {
            return Err(ContractError::NotAuthorizedMinter);
        }

        extend_instance_ttl(&env);

        receive_balance(&env, to.clone(), amount);

        TokenUtils::new(&env).events().mint(minter, to, amount);

        Ok(())
    }

    /// Transfers ownership of the interchain token to a new owner.
    ///
    /// Verifies authorization of the current owner, updates token ownership via `set_owner`,
    /// and emits a `transfer_ownership` event.
    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env);

        owner.require_auth();

        //        shared_interfaces::set_owner(&env, &new_owner);

        /*TokenUtils::new(&env)
                    .events()
                    .set_admin(owner.clone(), new_owner.clone());
        */
        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    pub fn add_minter(env: &Env, minter: Address) {
        Self::owner(&env).require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Minter(minter.clone()), &true);

        event::add_minter(env, minter);
    }
}

#[contractimpl]
impl token::Interface for InterchainToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        extend_instance_ttl(&env);
        read_allowance(&env, from, spender).amount
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        check_nonnegative_amount(amount);
        extend_instance_ttl(&env);

        write_allowance(
            &env,
            from.clone(),
            spender.clone(),
            amount,
            expiration_ledger,
        );

        TokenUtils::new(&env)
            .events()
            .approve(from, spender, amount, expiration_ledger);
    }

    fn balance(env: Env, id: Address) -> i128 {
        extend_instance_ttl(&env);
        read_balance(&env, id)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        extend_instance_ttl(&env);

        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);
        TokenUtils::new(&env).events().transfer(from, to, amount);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        extend_instance_ttl(&env);

        spend_allowance(&env, from.clone(), spender, amount);

        spend_balance(&env, from.clone(), amount);
        receive_balance(&env, to.clone(), amount);
        TokenUtils::new(&env).events().transfer(from, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        check_nonnegative_amount(amount);
        extend_instance_ttl(&env);

        spend_balance(&env, from.clone(), amount);
        TokenUtils::new(&env).events().burn(from, amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        check_nonnegative_amount(amount);
        extend_instance_ttl(&env);

        spend_allowance(&env, from.clone(), spender, amount);

        spend_balance(&env, from.clone(), amount);
        TokenUtils::new(&env).events().burn(from, amount)
    }

    fn decimals(env: Env) -> u32 {
        read_decimal(&env)
    }

    fn name(env: Env) -> String {
        read_name(&env)
    }

    fn symbol(env: Env) -> String {
        read_symbol(&env)
    }
}

impl InterchainToken {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl MigratableInterface for InterchainToken {
    type MigrationData = ();
    type Error = ContractError;

    fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
}

#[contractimpl]
impl UpgradableInterface for InterchainToken {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        shared_interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contractimpl]
impl OwnableInterface for InterchainToken {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        shared_interfaces::owner(env)
    }
}
