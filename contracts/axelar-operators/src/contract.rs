use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Val, Vec};

use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_interfaces::axelar_operators::{AxelarOperatorsInterface, OperatorError};

#[contract]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), OperatorError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(OperatorError::NotInitialized)?;

        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
        Ok(())
    }

    pub fn owner(env: &Env) -> Result<Address, OperatorError> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(OperatorError::NotInitialized)
    }
}

#[contractimpl]
impl AxelarOperatorsInterface for AxelarOperators {
    fn initialize(env: Env, owner: Address) -> Result<(), OperatorError> {
        if env
            .storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
            .is_some()
        {
            Err(OperatorError::AlreadyInitialized)?
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Owner, &owner);
        Ok(())
    }

    fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().persistent().has(&key)
    }

    fn add_operator(env: Env, account: Address) -> Result<(), OperatorError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(OperatorError::NotInitialized)?;

        owner.require_auth();

        let key = DataKey::Operators(account.clone());

        if env.storage().persistent().has(&key) {
            Err(OperatorError::OperatorAlreadyAdded)?
        }

        env.storage().persistent().set(&key, &true);

        event::add_operator(&env, account);
        Ok(())
    }

    fn remove_operator(env: Env, account: Address) -> Result<(), OperatorError> {
        let owner: Address = env
            .storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(OperatorError::NotInitialized)?;

        owner.require_auth();

        let key = DataKey::Operators(account.clone());

        if !env.storage().persistent().has(&key) {
            Err(OperatorError::NotAnOperator)?
        }

        env.storage().persistent().remove(&key);

        event::remove_operator(&env, account);
        Ok(())
    }

    fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, OperatorError> {
        operator.require_auth();

        let key = DataKey::Operators(operator.clone());

        if !env.storage().persistent().has(&key) {
            Err(OperatorError::NotAnOperator)?
        }

        let res: Val = env.invoke_contract(&contract, &func, args);

        Ok(res)
    }
}
