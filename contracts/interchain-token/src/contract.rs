use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(env: Env,
        owner: Address,
        interchain_token_service: Address,
        minter: Address,
        token_id: Bytes,
        token_meta_data: TokenMetadata,) {
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

    pub fn owner(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("owner not found")
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner: Address = Self::owner(&env);
        owner.require_auth();

        shared_interfaces::set_owner(&env, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
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
