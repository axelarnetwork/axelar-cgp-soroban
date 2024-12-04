use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_std::interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};
use axelar_soroban_std::{ensure, interfaces};
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Val, Vec};

#[contract]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn __constructor(env: Env, owner: Address) {
        interfaces::set_owner(&env, &owner);
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env);

        owner.require_auth();

        interfaces::set_owner(&env, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    /// Return true if the account is an operator.
    pub fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().instance().has(&key)
    }

    /// Add an address as an operator.
    ///
    /// Only callable by the contract owner.
    pub fn add_operator(env: Env, account: Address) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::Operators(account.clone());

        ensure!(
            !env.storage().instance().has(&key),
            ContractError::OperatorAlreadyAdded
        );

        env.storage().instance().set(&key, &true);

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
            env.storage().instance().has(&key),
            ContractError::NotAnOperator
        );

        env.storage().instance().remove(&key);

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
            env.storage().instance().has(&key),
            ContractError::NotAnOperator
        );

        let res: Val = env.invoke_contract(&contract, &func, args);

        Ok(res)
    }
}

impl AxelarOperators {
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl MigratableInterface for AxelarOperators {
    type MigrationData = ();
    type Error = ContractError;

    fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
}

#[contractimpl]
impl UpgradableInterface for AxelarOperators {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contractimpl]
impl OwnableInterface for AxelarOperators {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        interfaces::owner(env)
    }
}
