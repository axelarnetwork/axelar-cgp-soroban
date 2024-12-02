use axelar_soroban_std::constants::{INSTANCE_TTL_EXTEND_TO, INSTANCE_TTL_THRESHOLD};
use soroban_sdk::token::{self, Interface as _};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

use crate::storage_types::{AllowanceDataKey, AllowanceValue};

use axelar_soroban_std::shared_interfaces::{
    migrate, MigratableInterface, OwnableInterface, UpgradableInterface,
};
use axelar_soroban_std::{assert_with_error, ensure, shared_interfaces};

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String};

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(
        env: Env,
        owner: Address,
        minter: Address,
        interchain_token_service: Address,
        token_id: BytesN<32>,
        token_meta_data: TokenMetadata,
    ) -> Result<(), ContractError> {
        shared_interfaces::set_owner(&env, &owner);

        Self::validate_token_metadata(token_meta_data.clone())?;

        env.storage().instance().set(&DataKey::TokenId, &token_id);

        Self::write_metadata(&env, token_meta_data);

        env.storage()
            .persistent()
            .set(&DataKey::Minter(minter), &());
        env.storage()
            .persistent()
            .set(&DataKey::Minter(interchain_token_service.clone()), &());
        env.storage()
            .instance()
            .set(&DataKey::InterchainTokenService, &interchain_token_service);

        Ok(())
    }

    pub fn token_id(env: &Env) -> BytesN<32> {
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

    pub fn is_minter(env: &Env, minter: Address) -> bool {
        env.storage().persistent().has(&DataKey::Minter(minter))
    }

    pub fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        minter.require_auth();

        ensure!(
            Self::is_minter(&env, minter.clone()),
            ContractError::NotMinter
        );

        Self::check_nonnegative_amount(amount);

        Self::extend_instance_ttl(&env);

        Self::receive_balance(&env, to.clone(), amount);

        TokenUtils::new(&env).events().mint(minter, to, amount);

        Ok(())
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env);

        owner.require_auth();

        shared_interfaces::set_owner(&env, &new_owner);

        TokenUtils::new(&env)
            .events()
            .set_admin(owner.clone(), new_owner.clone());

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    pub fn add_minter(env: &Env, minter: Address) {
        Self::owner(env).require_auth();

        env.storage()
            .persistent()
            .set(&DataKey::Minter(minter.clone()), &());

        event::add_minter(env, minter);
    }

    pub fn remove_minter(env: &Env, minter: Address) {
        Self::owner(env).require_auth();

        env.storage()
            .persistent()
            .remove(&DataKey::Minter(minter.clone()));

        event::remove_minter(env, minter);
    }
}

#[contractimpl]
impl token::Interface for InterchainToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        Self::extend_instance_ttl(&env);
        Self::read_allowance(&env, from, spender).amount
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        Self::check_nonnegative_amount(amount);
        Self::extend_instance_ttl(&env);

        Self::write_allowance(
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
        Self::extend_instance_ttl(&env);
        Self::read_balance(&env, id)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        Self::check_nonnegative_amount(amount);
        Self::extend_instance_ttl(&env);

        Self::spend_balance(&env, from.clone(), amount);
        Self::receive_balance(&env, to.clone(), amount);
        TokenUtils::new(&env).events().transfer(from, to, amount);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        Self::check_nonnegative_amount(amount);
        Self::extend_instance_ttl(&env);

        Self::spend_allowance(&env, from.clone(), spender, amount);

        Self::spend_balance(&env, from.clone(), amount);
        Self::receive_balance(&env, to.clone(), amount);
        TokenUtils::new(&env).events().transfer(from, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        Self::check_nonnegative_amount(amount);
        Self::extend_instance_ttl(&env);

        Self::spend_balance(&env, from.clone(), amount);
        TokenUtils::new(&env).events().burn(from, amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        Self::check_nonnegative_amount(amount);
        Self::extend_instance_ttl(&env);

        Self::spend_allowance(&env, from.clone(), spender, amount);

        Self::spend_balance(&env, from.clone(), amount);
        TokenUtils::new(&env).events().burn(from, amount)
    }

    fn decimals(env: Env) -> u32 {
        TokenUtils::new(&env).metadata().get_metadata().decimal
    }

    fn name(env: Env) -> String {
        TokenUtils::new(&env).metadata().get_metadata().name
    }

    fn symbol(env: Env) -> String {
        TokenUtils::new(&env).metadata().get_metadata().symbol
    }
}

impl InterchainToken {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}

    const fn check_nonnegative_amount(amount: i128) {
        assert_with_error!(amount >= 0, "negative amount is not allowed");
    }

    fn validate_token_metadata(token_meta_data: TokenMetadata) -> Result<(), ContractError> {
        ensure!(
            token_meta_data.decimal <= u8::MAX.into(),
            ContractError::InvalidDecimal
        );
        ensure!(
            !token_meta_data.name.is_empty(),
            ContractError::InvalidTokenName
        );
        ensure!(
            !token_meta_data.symbol.is_empty(),
            ContractError::InvalidTokenSymbol
        );
        Ok(())
    }

    fn extend_instance_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    fn extend_balance_ttl(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    fn read_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
        let key = DataKey::Allowance(AllowanceDataKey { from, spender });
        env.storage()
            .temporary()
            .get::<_, AllowanceValue>(&key)
            .map_or(
                AllowanceValue {
                    amount: 0,
                    expiration_ledger: 0,
                },
                |allowance| {
                    if allowance.expiration_ledger < env.ledger().sequence() {
                        AllowanceValue {
                            amount: 0,
                            expiration_ledger: allowance.expiration_ledger,
                        }
                    } else {
                        allowance
                    }
                },
            )
    }

    fn write_allowance(
        env: &Env,
        from: Address,
        spender: Address,
        amount: i128,
        expiration_ledger: u32,
    ) {
        let allowance = AllowanceValue {
            amount,
            expiration_ledger,
        };

        assert_with_error!(
            !(amount > 0 && expiration_ledger < env.ledger().sequence()),
            "expiration_ledger is less than ledger seq when amount > 0"
        );

        let key = DataKey::Allowance(AllowanceDataKey { from, spender });
        env.storage().temporary().set(&key, &allowance);

        if amount > 0 {
            let live_for = expiration_ledger
                .checked_sub(env.ledger().sequence())
                .unwrap();

            env.storage()
                .temporary()
                .extend_ttl(&key, live_for, live_for)
        }
    }

    fn spend_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
        let allowance = Self::read_allowance(env, from.clone(), spender.clone());

        assert_with_error!(allowance.amount >= amount, "insufficient allowance");

        if amount > 0 {
            Self::write_allowance(
                env,
                from,
                spender,
                allowance
                    .amount
                    .checked_sub(amount)
                    .expect("insufficient allowance"),
                allowance.expiration_ledger,
            );
        }
    }

    // ahram: if we add has_check, this won't work
    fn read_balance(env: &Env, addr: Address) -> i128 {
        let key = DataKey::Balance(addr);
        env.storage()
            .persistent()
            .get::<DataKey, i128>(&key)
            .map_or(0, |balance| {
                Self::extend_balance_ttl(env, &key);
                balance
            })
    }

    fn receive_balance(env: &Env, addr: Address, amount: i128) {
        let key = DataKey::Balance(addr);

        env.storage()
            .persistent()
            .update(&key, |balance: Option<i128>| {
                balance.unwrap_or_default() + amount
            });
    }

    fn spend_balance(env: &Env, addr: Address, amount: i128) {
        let balance = Self::read_balance(env, addr.clone());

        assert_with_error!(balance >= amount, "insufficient balance");

        Self::write_balance(env, addr, balance - amount);
    }

    fn write_metadata(env: &Env, metadata: TokenMetadata) {
        TokenUtils::new(env).metadata().set_metadata(&metadata);
    }

    fn write_balance(env: &Env, addr: Address, amount: i128) {
        let key = DataKey::Balance(addr);
        env.storage().persistent().set(&key, &amount);
        Self::extend_balance_ttl(env, &key);
    }
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
