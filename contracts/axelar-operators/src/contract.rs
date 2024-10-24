use axelar_soroban_std::ensure;
use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)?;

        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    pub fn owner(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)
    }
}

#[contractimpl]
impl AxelarOperators {
    /// Initialize the operators contract with an owner.
    pub fn initialize(env: Env, owner: Address) -> Result<(), ContractError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            ContractError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Owner, &owner);
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
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)?;

        owner.require_auth();

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
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)?;

        owner.require_auth();

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

        let key = DataKey::Operators(operator.clone());

        ensure!(
            env.storage().persistent().has(&key),
            ContractError::NotAnOperator
        );

        let res: Val = env.invoke_contract(&contract, &func, args);

        Ok(res)
    }
}
