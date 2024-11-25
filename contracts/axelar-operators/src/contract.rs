use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_std::ownership::OwnershipInterface;
use axelar_soroban_std::upgrade::{standardized_migrate, UpgradeableInterface};
use axelar_soroban_std::{ensure, ownership, upgrade};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Symbol, Val, Vec};

#[contract]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    /// Initialize the operators contract with an owner.
    pub fn __constructor(env: Env, owner: Address) {
        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &owner);
    }

    pub fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        standardized_migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env);

        owner.require_auth();

        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    /// Return true if the account is an operator.
    pub fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().persistent().has(&key)
    }

    /// Add an address as an operator.
    ///
    /// Only callable by the contract owner.
    pub fn add_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            !env.storage().persistent().has(&key),
            ContractError::OperatorAlreadyAdded
        );

        env.storage().persistent().set(&key, &true);

        event::add_operator(&env, account);
        Ok(())
    }

    /// Remove an address as an operator.
    ///
    /// Only callable by the contract owner.
    pub fn remove_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            env.storage().persistent().has(&key),
            ContractError::NotAnOperator
        );

        env.storage().persistent().remove(&key);

        event::remove_operator(&env, account);
        Ok(())
    }

    /// Execute a function on a contract as an operator.
    pub fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, ContractError> {
        operator.require_auth();

        let key = DataKey::Operators(operator);

        ensure!(
            env.storage().persistent().has(&key),
            ContractError::NotAnOperator
        );

        let res: Val = env.invoke_contract(&contract, &func, args);

        Ok(res)
    }

    // Modify this function to add migration logic
    #[allow(clippy::missing_const_for_fn)] // exclude no-op implementations from this lint
    fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl UpgradeableInterface for AxelarOperators {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }
}

#[contractimpl]
impl OwnershipInterface for AxelarOperators {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        ownership::default_owner_impl(env)
    }
}
