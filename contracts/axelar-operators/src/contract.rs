use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Symbol, Val, Vec};

use crate::storage_types::DataKey;
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_operators::AxelarOperatorsInterface;

#[contract]
pub struct AxelarOperators;

#[contractimpl]
impl AxelarOperators {
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }

    pub fn owner(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }
}

#[contractimpl]
impl AxelarOperatorsInterface for AxelarOperators {
    fn initialize(env: Env, owner: Address) {
        if env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
        {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    fn is_operator(env: Env, account: Address) -> bool {
        let key = DataKey::Operators(account);

        env.storage().persistent().has(&key)
    }

    fn add_operator(env: Env, account: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        let key = DataKey::Operators(account.clone());

        if env.storage().persistent().has(&key) {
            panic_with_error!(env, Error::OperatorAlreadyAdded);
        }

        env.storage().persistent().set(&key, &true);

        event::add_operator(&env, account);
    }

    fn remove_operator(env: Env, account: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        let key = DataKey::Operators(account.clone());

        if !env.storage().persistent().has(&key) {
            panic_with_error!(env, Error::NotAnOperator);
        }

        env.storage().persistent().remove(&key);

        event::remove_operator(&env, account);
    }

    fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        func: Symbol, args:
        Vec<Val>
    ) -> Val {
        operator.require_auth();

        let key = DataKey::Operators(operator.clone());

        if !env.storage().persistent().has(&key) {
            panic_with_error!(env, Error::NotAnOperator);
        }

        let res: Val = env.invoke_contract(&contract, &func, args);

        res
    }
}
