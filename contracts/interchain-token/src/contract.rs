use axelar_soroban_std::ensure;
use soroban_sdk::token::{self, Interface as _};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use crate::utils::{
    admin, check_nonnegative_amount, extend_instance_ttl, read_allowance, read_balance,
    read_decimal, read_name, read_symbol, receive_balance, spend_allowance, spend_balance,
    write_allowance, write_metadata,
};

use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, IntoVal, String};

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(
        env: Env,
        interchain_token_service: Address,
        owner: Address,
        minter: Address,
        token_id: Bytes,
        token_meta_data: TokenMetadata,
    ) -> Result<(), ContractError> {
        env.storage().instance().set(&DataKey::Admin, &owner);

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

    pub fn set_admin(env: Env, new_admin: Address) {
        let admin = admin(&env);
        admin.require_auth();

        extend_instance_ttl(&env);

        env.storage().instance().set(&DataKey::Admin, &admin);

        TokenUtils::new(&env)
            .events()
            .set_admin(admin.clone(), new_admin.clone());

        event::set_admin(&env, admin, new_admin);
    }

    pub fn add_minter(env: &Env, minter: Address) {
        admin(env).require_auth();

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
