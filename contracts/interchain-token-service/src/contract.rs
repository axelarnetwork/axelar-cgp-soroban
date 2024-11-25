use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_std::ownership::OwnershipInterface;
use axelar_soroban_std::upgrade::{standardized_migrate, UpgradeableInterface};
use axelar_soroban_std::{ensure, ownership, upgrade};
use soroban_sdk::{contract, contractimpl, Address, Env, String};

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
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
        let owner = Self::owner(&env);
        owner.require_auth();

        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }

    pub fn trusted_address(env: &Env, chain: String) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::TrustedAddress(chain))
    }

    pub fn set_trusted_address(
        env: Env,
        chain: String,
        address: String,
    ) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let key = DataKey::TrustedAddress(chain.clone());

        ensure!(
            !env.storage().persistent().has(&key),
            ContractError::TrustedAddressAlreadySet
        );

        env.storage().persistent().set(&key, &address);

        event::set_trusted_address(&env, chain, address);

        Ok(())
    }

    pub fn remove_trusted_address(env: Env, chain: String) -> Result<(), ContractError> {
        Self::owner(&env).require_auth();

        let Some(trusted_address) = Self::trusted_address(&env, chain.clone()) else {
            return Err(ContractError::NoTrustedAddressSet);
        };

        env.storage()
            .persistent()
            .remove(&DataKey::TrustedAddress(chain.clone()));

        event::remove_trusted_address(&env, chain, trusted_address);

        Ok(())
    }
}

impl InterchainTokenService {
    // Modify this function to add migration logic
    #[allow(clippy::missing_const_for_fn)] // exclude no-op implementations from this lint
    fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl UpgradeableInterface for InterchainTokenService {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }
}

#[contractimpl]
impl OwnershipInterface for InterchainTokenService {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        ownership::default_owner_impl(env)
    }
}
