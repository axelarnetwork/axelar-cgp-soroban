use axelar_soroban_std::ensure;
use axelar_soroban_std::types::Token;
use soroban_sdk::{bytes, contract, contractimpl, Address, Bytes, Env, String};

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;
use crate::types::MessageType;

use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::AxelarGatewayMessagingClient;

use axelar_gateway::executable::AxelarExecutableInterface;

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl AxelarExecutableInterface for InterchainTokenService {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Gateway).unwrap()
    }

    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) {
        let _ = Self::validate_message(&env, &source_chain, &message_id, &source_address, &payload);

        let _ = Self::execute_message(
            &env,
            source_chain.clone(),
            message_id.clone(),
            source_address.clone(),
            payload.clone(),
        );

        event::executed(&env, source_chain, message_id, source_address, payload);
    }
}

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(env: Env, owner: Address, gateway: Address, gas_service: Address) {
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
    }

    pub fn owner(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("owner not found")
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

    pub fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    pub fn execute_message(
        _env: &Env,
        _source_chain: String,
        _message_id: String,
        _source_address: String,
        _payload: Bytes,
    ) -> Result<(), ContractError> {
        // TODO: get_execute_params

        let message_type = MessageType::DeployInterchainToken;

        match message_type {
            MessageType::InterchainTransfer => {
                // TODO
                Ok(())
            }
            MessageType::DeployInterchainToken => {
                // TODO
                Ok(())
            }
            MessageType::DeployTokenManager => {
                // Note: this case is not supported by the ITS hub
                Ok(())
            }
            _ => Err(ContractError::InvalidMessageType),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn deploy_interchain_token(
        _env: Env,
        _caller: Address,
        _destination_chain: String,
        _name: String,
        _symbol: String,
        _decimals: i128,
        _minter: Bytes,
        _gas_token: Token,
    ) {
        // TODO: deploy interchain token
    }

    pub fn deploy_remote_interchain_token(
        env: Env,
        caller: Address,
        destination_chain: String,
        _token_id: String,
        gas_token: Token,
        _metadata: Bytes,
    ) {
        let destination_address = String::from_str(&env, "destination_address");

        // TODO: abi encode with MessageType.DeployInterchainToken
        let payload = bytes!(&env, 0x1234);

        Self::pay_gas_and_call_contract(
            env,
            caller,
            destination_chain,
            destination_address,
            payload,
            gas_token,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn interchain_token_transfer(
        env: Env,
        caller: Address,
        _token_id: String,
        destination_chain: String,
        destination_address: String,
        _amount: i128,
        _metadata: Bytes,
        gas_token: Token,
    ) {
        // TODO: _takeToken, decode metadata, and abi encode with MessageType.InterchainTransfer
        let payload = bytes!(&env, 0x1234);

        Self::pay_gas_and_call_contract(
            env,
            caller,
            destination_chain,
            destination_address,
            payload,
            gas_token,
        );
    }

    fn pay_gas_and_call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        gas_token: Token,
    ) {
        let gateway = AxelarGatewayMessagingClient::new(&env, &Self::gateway(&env));
        let gas_service = AxelarGasServiceClient::new(&env, &Self::gas_service(&env));

        caller.require_auth();

        // get_call_aprams from payload

        gas_service.pay_gas_for_contract_call(
            &caller,
            &destination_chain,
            &destination_address,
            &payload,
            &caller,
            &gas_token,
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &payload,
        );
    }
}
