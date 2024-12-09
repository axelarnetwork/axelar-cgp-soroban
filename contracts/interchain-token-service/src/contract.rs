use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::{executable::AxelarExecutableInterface, AxelarGatewayMessagingClient};
use axelar_soroban_std::interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};
use axelar_soroban_std::{ensure, interfaces, types::Token};
use interchain_token::InterchainTokenClient;
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
const PREFIX_INTERCHAIN_TOKEN_ID: &str = "its-interchain-token-id";
const PREFIX_INTERCHAIN_TOKEN_SALT: &str = "interchain-token-salt";

#[contract]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(
        env: Env,
        owner: Address,
        gateway: Address,
        gas_service: Address,
        chain_name: String,
        interchain_token_wasm_hash: BytesN<32>,
    ) {
        interfaces::set_owner(&env, &owner);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
        env.storage()
            .instance()
            .set(&DataKey::ChainName, &chain_name);
        env.storage().instance().set(
            &DataKey::InterchainTokenWasmHash,
            &interchain_token_wasm_hash,
        );
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn chain_name(env: &Env) -> String {
        env.storage().instance().get(&DataKey::ChainName).unwrap()
    }

    fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenWasmHash)
            .expect("interchain token wasm hash not found")
    }

    fn its_hub_routing_identifier(env: &Env) -> String {
        String::from_str(env, ITS_HUB_ROUTING_IDENTIFIER)
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
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

    fn interchain_token_deploy_salt(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32> {
        let chain_name = Self::chain_name(env);
        let chain_name_hash: BytesN<32> = env.crypto().keccak256(&(chain_name).to_xdr(env)).into();
        env.crypto()
            .keccak256(
                &(
                    PREFIX_INTERCHAIN_TOKEN_SALT,
                    chain_name_hash,
                    deployer,
                    salt,
                )
                    .to_xdr(env),
            )
            .into()
    }

    fn interchain_token_id(env: &Env, sender: Option<Address>, salt: BytesN<32>) -> BytesN<32> {
        let value = match sender {
            Some(sender) => (PREFIX_INTERCHAIN_TOKEN_ID, sender, salt).to_xdr(env),
            None => (salt).to_xdr(env),
        };
        env.crypto().keccak256(&value).into()
    }

    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        token_meta_data: TokenMetadata,
        initial_supply: i128,
        minter: Option<Address>,
    ) -> Result<(Address, BytesN<32>), ContractError> {
        caller.require_auth();

        let initial_minter = if initial_supply > 0 {
            Some(env.current_contract_address())
        } else if let Some(ref minter) = minter {
            ensure!(
                *minter != env.current_contract_address(),
                ContractError::InvalidMinter
            );
            Some(minter.clone())
        } else {
            None
        };

        let deploy_salt = Self::interchain_token_deploy_salt(env, caller.clone(), salt);
        let token_id = Self::interchain_token_id(env, None, deploy_salt);

        let deployed_address = env
            .deployer()
            .with_address(env.current_contract_address(), token_id.clone())
            .deploy_v2(
                Self::interchain_token_wasm_hash(env),
                (
                    env.current_contract_address(),
                    initial_minter,
                    env.current_contract_address(),
                    token_id.clone(),
                    token_meta_data,
                ),
            );

        if initial_supply > 0 {
            let token = InterchainTokenClient::new(env, &deployed_address);

            // AXE-6858: the tokenManager related logic needs to be implemented here.
            token.mint(&env.current_contract_address(), &caller, &initial_supply);

            if let Some(minter) = minter {
                token.remove_minter(&env.current_contract_address());
                token.add_minter(&minter);
            }
        }

        Ok((deployed_address, token_id))
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
        env.storage()
            .instance()
            .get(&DataKey::Gateway)
            .expect("gateway not found")
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
    // Modify this function to add migration logic
    const fn run_migration(_env: &Env, _migration_data: ()) {}

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
        let message_type = get_message_type(&payload.to_alloc_vec())?;

        ensure!(
            message_type == EncodedMessageType::ReceiveFromHub,
            ContractError::InvalidMessageType
        );

        ensure!(
            source_chain == Self::its_hub_chain_name(env),
            ContractError::UntrustedChain
        );

        let decoded_message = HubMessage::abi_decode(env, payload)?;

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
        .abi_encode(env)?;

        Ok(payload)
    }
}

#[contractimpl]
impl MigratableInterface for InterchainTokenService {
    type MigrationData = ();
    type Error = ContractError;

    fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
}

#[contractimpl]
impl UpgradableInterface for InterchainTokenService {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contractimpl]
impl OwnableInterface for InterchainTokenService {
    fn owner(env: &Env) -> Address {
        interfaces::owner(env)
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        interfaces::transfer_ownership::<Self>(env, new_owner);
    }
}
