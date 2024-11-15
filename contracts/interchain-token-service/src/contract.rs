use axelar_soroban_std::ensure;
use soroban_sdk::{contract, contractimpl, Address, Env, String};

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(env: Env, owner: Address) {
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    pub fn owner(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).expect("owner not found")
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner = Self::owner(&env);
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

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
