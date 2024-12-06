use axelar_soroban_std::ttl::{
    extend_instance_ttl, INSTANCE_TTL_EXTEND_TO, INSTANCE_TTL_THRESHOLD,
};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

use crate::interface::InterchainTokenInterface;
use crate::storage_types::{AllowanceDataKey, AllowanceValue};
use axelar_soroban_std::interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};
use axelar_soroban_std::{ensure, interfaces};
use soroban_sdk::token::TokenInterface;

use soroban_sdk::{assert_with_error, contract, contractimpl, token, Address, BytesN, Env, String};
use soroban_token_sdk::event::Events as TokenEvents;

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
        interfaces::set_owner(&env, &owner);

        Self::validate_token_metadata(token_meta_data.clone())?;

        Self::write_metadata(&env, token_meta_data);

        env.storage().instance().set(&DataKey::TokenId, &token_id);
        env.storage().instance().set(&DataKey::Minter(minter), &());
        env.storage()
            .instance()
            .set(&DataKey::Minter(interchain_token_service.clone()), &());
        env.storage()
            .instance()
            .set(&DataKey::InterchainTokenService, &interchain_token_service);

        Ok(())
    }
}

#[contractimpl]
impl InterchainTokenInterface for InterchainToken {
    fn token_id(env: &Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::TokenId)
            .expect("token id not found")
    }

    fn interchain_token_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenService)
            .expect("interchain token service not found")
    }

    fn is_minter(env: &Env, minter: Address) -> bool {
        env.storage().instance().has(&DataKey::Minter(minter))
    }

    fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), ContractError> {
        minter.require_auth();

        ensure!(
            Self::is_minter(&env, minter.clone()),
            ContractError::NotMinter
        );

        Self::validate_amount(&env, amount);

        extend_instance_ttl(&env);

        Self::receive_balance(&env, to.clone(), amount);

        TokenUtils::new(&env).events().mint(minter, to, amount);

        Ok(())
    }

    fn add_minter(env: &Env, minter: Address) {
        Self::owner(env).require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Minter(minter.clone()), &());

        event::add_minter(env, minter);
    }

    fn remove_minter(env: &Env, minter: Address) {
        Self::owner(env).require_auth();

        env.storage()
            .instance()
            .remove(&DataKey::Minter(minter.clone()));

        event::remove_minter(env, minter);
    }
}

#[contractimpl]
impl token::Interface for InterchainToken {
    fn allowance(env: Env, from: Address, spender: Address) -> i128 {
        extend_instance_ttl(&env);
        Self::read_allowance(&env, from, spender).amount
    }

    fn approve(env: Env, from: Address, spender: Address, amount: i128, expiration_ledger: u32) {
        from.require_auth();

        Self::validate_amount(&env, amount);
        extend_instance_ttl(&env);

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
        extend_instance_ttl(&env);
        Self::read_balance(&env, id)
    }

    fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        from.require_auth();

        extend_instance_ttl(&env);
        Self::validate_amount(&env, amount);
        Self::spend_balance(&env, from.clone(), amount);
        Self::receive_balance(&env, to.clone(), amount);

        TokenUtils::new(&env).events().transfer(from, to, amount);
    }

    fn transfer_from(env: Env, spender: Address, from: Address, to: Address, amount: i128) {
        spender.require_auth();

        extend_instance_ttl(&env);
        Self::validate_amount(&env, amount);
        Self::spend_allowance(&env, from.clone(), spender, amount);
        Self::spend_balance(&env, from.clone(), amount);
        Self::receive_balance(&env, to.clone(), amount);

        TokenUtils::new(&env).events().transfer(from, to, amount)
    }

    fn burn(env: Env, from: Address, amount: i128) {
        from.require_auth();

        extend_instance_ttl(&env);
        Self::validate_amount(&env, amount);
        Self::spend_balance(&env, from.clone(), amount);

        TokenUtils::new(&env).events().burn(from, amount);
    }

    fn burn_from(env: Env, spender: Address, from: Address, amount: i128) {
        spender.require_auth();

        extend_instance_ttl(&env);
        Self::validate_amount(&env, amount);
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

    fn validate_amount(env: &Env, amount: i128) {
        assert_with_error!(env, amount >= 0, ContractError::InvalidAmount);
    }

    fn validate_token_metadata(
        TokenMetadata {
            decimal,
            name,
            symbol,
        }: TokenMetadata,
    ) -> Result<(), ContractError> {
        ensure!(decimal <= u8::MAX.into(), ContractError::InvalidDecimal);
        ensure!(!name.is_empty(), ContractError::InvalidTokenName);
        ensure!(!symbol.is_empty(), ContractError::InvalidTokenSymbol);
        Ok(())
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
            env,
            !(amount > 0 && expiration_ledger < env.ledger().sequence()),
            ContractError::InvalidExpirationLedger
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

        assert_with_error!(
            env,
            allowance.amount >= amount,
            ContractError::InsufficientAllowance
        );

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

    fn read_balance(env: &Env, addr: Address) -> i128 {
        let key = DataKey::Balance(addr);
        env.storage()
            .persistent()
            .get::<_, i128>(&key)
            .inspect(|_| {
                // Extend the TTL of the balance entry when the balance is successfully retrieved.
                Self::extend_balance_ttl(env, &key);
            })
            .unwrap_or_default()
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

        assert_with_error!(env, balance >= amount, ContractError::InsufficientBalance);

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
        interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
}

#[contractimpl]
impl UpgradableInterface for InterchainToken {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contractimpl]
impl OwnableInterface for InterchainToken {
    fn owner(env: &Env) -> Address {
        interfaces::owner(env)
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        interfaces::transfer_ownership::<Self>(env, new_owner.clone());
        // adhere to reference implementation for tokens and emit predefined soroban event
        TokenEvents::new(env).set_admin(Self::owner(env), new_owner);
    }
}
