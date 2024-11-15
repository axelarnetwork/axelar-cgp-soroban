use axelar_soroban_std::ensure;
use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::event;
use crate::storage_types::DataKey;
use crate::utils::{
    admin, check_nonnegative_amount, extend_instance_ttl, read_allowance, read_balance,
    read_decimal, read_name, read_symbol, receive_balance, spend_allowance, spend_balance,
    write_allowance, write_metadata,
};

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn initialize_interchain_token(
        env: Env,
        interchain_token_service: Address,
        admin: Address,
        minter: Address,
        token_id: Bytes,
        token_meta_data: TokenMetadata,
    ) -> Result<(), ContractError> {
        ensure!(!token_id.is_empty(), ContractError::TokenIdZero);
        ensure!(token_meta_data.decimal <= 18, ContractError::InvalidDecimal);
        ensure!(
            !token_meta_data.name.is_empty(),
            ContractError::TokenNameEmpty
        );
        ensure!(
            !token_meta_data.symbol.is_empty(),
            ContractError::TokenSymbolEmpty
        );

        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            ContractError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::TokenId, &token_id);
        write_metadata(&env, token_meta_data);

        env.storage().instance().set(&DataKey::Owner, &owner);

        Ok(())
    }

    pub fn owner(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env)?;
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

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
