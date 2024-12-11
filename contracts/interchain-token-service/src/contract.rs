use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::{executable::AxelarExecutableInterface, AxelarGatewayMessagingClient};
use axelar_soroban_std::interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};
use axelar_soroban_std::{ensure, interfaces, types::Token};
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
        its_hub_address: String,
        chain_name: String,
    ) {
        interfaces::set_owner(&env, &owner);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
        env.storage()
            .instance()
            .set(&DataKey::ItsHubAddress, &its_hub_address);
        env.storage()
            .instance()
            .set(&DataKey::ChainName, &chain_name);
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn chain_name(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ChainName)
            .expect("chain name not found")
    }

    fn gas_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasService)
            .expect("gas service not found")
    }

    fn its_hub_address(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ItsHubAddress)
            .expect("its hub address not found")
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
    }

    fn is_trusted_chain(env: &Env, chain: String) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::TrustedChain(chain))
    }

    fn set_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let key = DataKey::TrustedChain(chain.clone());

        ensure!(
            !env.storage().persistent().has(&key),
            ContractError::TrustedChainAlreadySet
        );

        env.storage().persistent().set(&key, &true);

        event::set_trusted_chain(env, chain);

        Ok(())
    }

    fn remove_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        Self::owner(env).require_auth();

        let key = DataKey::TrustedChain(chain.clone());

        ensure!(
            env.storage().persistent().has(&key),
            ContractError::TrustedChainNotSet
        );

        env.storage().persistent().remove(&key);

        event::remove_trusted_chain(env, chain);

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
        let hub_chain = Self::its_hub_chain_name(env);
        let hub_address = Self::its_hub_address(env);

        gas_service.pay_gas(
            &env.current_contract_address(),
            &hub_chain,
            &hub_address,
            &payload,
            &caller,
            &gas_token,
            &Bytes::new(env),
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &hub_chain,
            &hub_address,
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

        ensure!(
            Self::is_trusted_chain(env, original_source_chain.clone()),
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
            Self::is_trusted_chain(env, destination_chain.clone()),
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
