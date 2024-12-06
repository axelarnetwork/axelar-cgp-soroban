use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::{executable::AxelarExecutableInterface, AxelarGatewayMessagingClient};
use axelar_soroban_std::{ensure, types::Token};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::abi::{get_message_type, MessageType as EncodedMessageType};
use crate::error::ContractError;
use crate::event;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage_types::DataKey;
use crate::types::{HubMessage, InterchainTransfer, Message};

const ITS_HUB_CHAIN_NAME: &str = "axelar";
const ITS_HUB_ROUTING_IDENTIFIER: &str = "hub";

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(env: Env, owner: Address, gateway: Address, gas_service: Address) {
        env.storage().instance().set(&DataKey::Owner, &owner);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    fn its_hub_routing_identifier(env: &Env) -> String {
        String::from_str(env, ITS_HUB_ROUTING_IDENTIFIER)
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
    }

    fn owner(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("owner not found")
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        let owner = Self::owner(env);
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(env, owner, new_owner);
    }

    fn trusted_address(env: &Env, chain: String) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::TrustedAddress(chain))
    }

    fn set_trusted_address(env: &Env, chain: String, address: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let key = DataKey::TrustedAddress(chain.clone());

        ensure!(
            !env.storage().persistent().has(&key),
            ContractError::TrustedAddressAlreadySet
        );

        env.storage().persistent().set(&key, &address);

        event::set_trusted_address(env, chain, address);

        Ok(())
    }

    fn remove_trusted_address(env: &Env, chain: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let Some(trusted_address) = Self::trusted_address(env, chain.clone()) else {
            return Err(ContractError::NoTrustedAddressSet);
        };

        env.storage()
            .persistent()
            .remove(&DataKey::TrustedAddress(chain.clone()));

        event::remove_trusted_address(env, chain, trusted_address);

        Ok(())
    }

    fn deploy_interchain_token(
        _env: &Env,
        _caller: Address,
        _salt: BytesN<32>,
        _destination_chain: String,
        _token_metadata: TokenMetadata,
        _minter: Option<Bytes>,
        _gas_token: Token,
    ) -> Result<BytesN<32>, ContractError> {
        todo!()
    }

    fn deploy_remote_interchain_token(
        _env: &Env,
        _caller: Address,
        _salt: BytesN<32>,
        _minter: Option<Bytes>,
        _destination_chain: String,
        _gas_token: Token,
    ) -> Result<BytesN<32>, ContractError> {
        // TODO: implementation
        // deploy salt
        // minter approval
        // registered token address - get token metadata
        // abi encode with MessageType.DeployInterchainToken
        // token id
        // pay_gas_and_call_contract
        todo!()
    }

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        metadata: Option<Bytes>,
        gas_token: Token,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        // TODO: implementation

        let message = Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: caller.clone().to_xdr(env),
            destination_address,
            amount,
            data: metadata,
        });

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(())
    }
}

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

impl InterchainTokenService {
    fn pay_gas_and_call_contract(
        env: &Env,
        caller: Address,
        destination_chain: String,
        message: Message,
        gas_token: Token,
    ) -> Result<(), ContractError> {
        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));
        let gas_service = AxelarGasServiceClient::new(env, &Self::gas_service(env));

        let payload = Self::get_call_params(env, destination_chain, message)?;

        let destination_address = Self::trusted_address(env, Self::its_hub_chain_name(env))
            .ok_or(ContractError::NoTrustedAddressSet)?;

        gas_service.pay_gas(
            &env.current_contract_address(),
            &Self::its_hub_chain_name(env),
            &destination_address,
            &payload,
            &caller,
            &gas_token,
            &Bytes::new(env),
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &Self::its_hub_chain_name(env),
            &destination_address,
            &payload,
        );

        Ok(())
    }

    fn execute_message(
        env: &Env,
        source_chain: String,
        _message_id: String,
        _source_address: String,
        payload: Bytes,
    ) -> Result<(), ContractError> {
        // TODO: Add ITS hub execute logic

        let (original_source_chain, message) =
            Self::get_execute_params(env, source_chain, &payload)?;

        match message {
            Message::InterchainTransfer(inner_message) => {
                // TODO: transfer implementation

                event::interchain_transfer_received(
                    env,
                    original_source_chain,
                    inner_message.token_id,
                    inner_message.source_address,
                    inner_message.destination_address,
                    inner_message.amount,
                    inner_message.data,
                );

                Ok(())
            }
            Message::DeployInterchainToken(_) => {
                // TODO
                Ok(())
            }
        }
    }

    fn get_execute_params(
        env: &Env,
        source_chain: String,
        payload: &Bytes,
    ) -> Result<(String, Message), ContractError> {
        let message_type =
            get_message_type(&payload.to_alloc_vec()).map_err(|_| ContractError::InvalidPayload)?;

        ensure!(
            message_type == EncodedMessageType::ReceiveFromHub,
            ContractError::InvalidMessageType
        );

        ensure!(
            source_chain == Self::its_hub_chain_name(env),
            ContractError::UntrustedChain
        );

        let decoded_message =
            HubMessage::abi_decode(env, payload).map_err(|_| ContractError::InvalidPayload)?;

        let HubMessage::ReceiveFromHub {
            source_chain: original_source_chain,
            message: inner_message,
        } = decoded_message
        else {
            return Err(ContractError::InvalidMessageType);
        };

        let trusted_address = Self::trusted_address(env, original_source_chain.clone());
        let routing_identifier = Self::its_hub_routing_identifier(env);

        ensure!(
            trusted_address.is_some_and(|addr| addr == routing_identifier),
            ContractError::UntrustedChain
        );

        Ok((original_source_chain, inner_message))
    }

    fn get_call_params(
        env: &Env,
        destination_chain: String,
        message: Message,
    ) -> Result<Bytes, ContractError> {
        // Note: ITS Hub chain as the actual destination chain for the messsage isn't supported
        ensure!(
            destination_chain != Self::its_hub_chain_name(env),
            ContractError::UntrustedChain
        );

        let Some(destination_address) = Self::trusted_address(env, destination_chain.clone())
        else {
            return Err(ContractError::UntrustedChain);
        };

        ensure!(
            destination_address == Self::its_hub_routing_identifier(env),
            ContractError::UntrustedChain
        );

        let payload = HubMessage::SendToHub {
            destination_chain,
            message,
        }
        .abi_encode(env)
        .map_err(|_| ContractError::InvalidPayload)?;

        Ok(payload)
    }
}
