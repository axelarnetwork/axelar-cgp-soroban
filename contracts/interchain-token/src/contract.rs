use crate::error::ContractError;
use crate::event;
use axelar_soroban_std::ownership::OwnershipInterface;
use axelar_soroban_std::upgrade::{standardized_migrate, UpgradeableInterface};
use axelar_soroban_std::{ownership, upgrade};
use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(env: Env, owner: Address) {
        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &owner);
    }

    pub fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        standardized_migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner: Address = Self::owner(&env);
        owner.require_auth();

        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }
}

impl InterchainToken {
    // Modify this function to add migration logic
    #[allow(clippy::missing_const_for_fn)] // exclude no-op implementations from this lint
    fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl UpgradeableInterface for InterchainToken {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }
}

#[contractimpl]
impl OwnershipInterface for InterchainToken {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        ownership::default_owner_impl(env)
    }
}
