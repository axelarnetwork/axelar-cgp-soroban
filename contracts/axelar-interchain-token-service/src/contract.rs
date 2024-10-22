use axelar_soroban_std::ensure;
use soroban_sdk::{contract, contractimpl, Address, Env, String,};

use crate::error::InterchainTokenServiceError;
use crate::event;
use crate::storage_types::{DataKey, TrustedItsAddress};

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn initialize(
        env: Env,
        owner: Address
    ) -> Result<(), InterchainTokenServiceError> {
        ensure!(
            env.storage()
                .instance()
                .get(&DataKey::Initialized)
                .unwrap_or(true),
            InterchainTokenServiceError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Owner, &owner);

        Ok(())
    }

    pub fn owner(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner = Self::owner(&env);
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }

    pub fn is_trusted_address(env: &Env, chain: String, address: String) -> bool {
        let key = DataKey::TrustedAddress(TrustedItsAddress { chain_name: chain, trusted_address: address });
        env.storage().persistent().has(&key)
    }

    pub fn set_trusted_address(
        env: Env, 
        chain: String, 
        address: String,
    ) -> Result<(), InterchainTokenServiceError> {
        Self::owner(&env).require_auth();

        let key = DataKey::TrustedAddress(
            TrustedItsAddress { 
                chain_name: chain.clone(), 
                trusted_address: address.clone()
            }
        );

        ensure!(
            !env.storage().persistent().has(&key),
            InterchainTokenServiceError::TrustedAddressAlreadyAdded
        );

        env.storage().persistent().set(&key, &true);

        event::set_trusted_address(&env, chain, address);
        Ok(())
    } 

    pub fn remove_trusted_address(
        env: Env, 
        chain: String, 
        address: String,
    ) -> Result<(), InterchainTokenServiceError> {
        Self::owner(&env).require_auth();

        let key = DataKey::TrustedAddress(
            TrustedItsAddress { 
                chain_name: chain.clone(), 
                trusted_address: address.clone()
            }
        );

        ensure!(
            env.storage().persistent().has(&key),
            InterchainTokenServiceError::NotTrustedAddress
        );

        env.storage().persistent().remove(&key);

        event::remove_trusted_address(&env, chain, address);
        Ok(())
    }
}

