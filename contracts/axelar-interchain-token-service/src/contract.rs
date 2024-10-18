use axelar_soroban_std::ensure;
use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec,
};

use crate::storage_types::{DataKey, TrustedItsAddress};
use crate::event;

use axelar_soroban_interfaces::interchain_token_service::{InterchainTokenServiceInterface, InterchainTokenServiceError};

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
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

        let owner = Self::owner(&env);
        owner.require_auth();

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

        let owner = Self::owner(&env);
        owner.require_auth();

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

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn initialize(
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

    fn interchain_token_id(
        env: Env,
        deployer: Address,
        salt: BytesN<32>,
    ) -> BytesN<32> {
        todo!("implement method");
    }

    fn valid_interchain_token_address(
        env: Env,
        token_id: BytesN<32>,
    ) -> BytesN<32> {
        todo!("implement method");
    }

    fn interchain_token_address(
        env: Env,
        token_id: BytesN<32>,
    ) -> BytesN<32> {
        todo!("implement method");
    }

    fn deploy_interchain_token(
        env: Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        name: String,
        symbol: String,
        decimals: u64, // should be u8 
        minter: Bytes,
    ) -> BytesN<32> {
        todo!("implement method and fix decimals parameter (change back to u8 as originally defined)");
    }

    fn interchain_transfer(
        env: Env,
        caller: Address,
        token_id: BytesN<32>,
        amount: i128,
        destination_chain: String,
        destination_address: String,
        metadata: Bytes,
    ) {
        todo!("implement method");
    }
}