use axelar_soroban_std::types::Token;
use axelar_soroban_std::{ensure, interfaces};

use interchain_token::InterchainTokenClient;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{bytes, contract, contractimpl, Address, Bytes, BytesN, Env, FromVal, String};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::error::ContractError;
use crate::event;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage_types::DataKey;
use crate::types::MessageType;

use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::AxelarGatewayMessagingClient;

use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};

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

    fn gas_service(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasService)
            .expect("gas service not found")
    }

    fn chain_name(env: &Env) -> String {
        env.storage()
            .instance()
            .get(&DataKey::ChainName)
            .expect("chain name not found")
    }

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32> {
        env.storage()
            .instance()
            .get(&DataKey::InterchainTokenWasmHash)
            .expect("interchain token wasm hash not found")
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

        // TODO: Add ITS hub routing logic

        gas_service.pay_gas(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &payload,
            &caller,
            &gas_token,
            &Bytes::new(&env),
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &payload,
        );
    }

    fn execute_message(
        _env: &Env,
        _source_chain: String,
        _message_id: String,
        _source_address: String,
        _payload: Bytes,
    ) -> Result<(), ContractError> {
        // TODO: Add ITS hub execute logic

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
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
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

    fn interchain_token_id(env: &Env, sender: Address, salt: BytesN<32>) -> BytesN<32> {
        env.crypto()
            .keccak256(&(PREFIX_INTERCHAIN_TOKEN_ID, sender, salt).to_xdr(env))
            .into()
    }

    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        token_meta_data: TokenMetadata,
        initial_supply: i128,
        _minter: Address,
    ) -> Result<BytesN<32>, ContractError> {
        caller.require_auth();

        let minter = if initial_supply > 0 {
            env.current_contract_address()
        } else {
            ensure!(
                _minter != env.current_contract_address(),
                ContractError::InvalidMinter
            );
            _minter
        };

        let deploy_salt = Self::interchain_token_deploy_salt(env, caller.clone(), salt.clone());
        let interchain_token_wasm_hash = Self::interchain_token_wasm_hash(env);
        let token_id = Self::interchain_token_id(env, caller.clone(), salt);

        let deployed_address = env
            .deployer()
            .with_address(caller.clone(), deploy_salt)
            .deploy_v2(
                interchain_token_wasm_hash,
                (
                    caller.clone(),
                    minter.clone(),
                    env.current_contract_address(),
                    token_id.clone(),
                    token_meta_data,
                ),
            );

        let token = InterchainTokenClient::new(env, &deployed_address);

        if initial_supply > 0 {
            // TODO: tokenManager, registeredTokenAddress, deployedTokenManager, removeFlowLimiter, addFlowLimiter
            token.mint(&minter, &caller, &initial_supply);
        }

        Ok(token_id)
    }

    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        destination_chain: String,
        _token_id: String,
        gas_token: Token,
    ) {
        let destination_address = String::from_str(env, "");

        // TODO: abi encode with MessageType.DeployInterchainToken
        let payload = bytes!(env,);

        Self::pay_gas_and_call_contract(
            env.clone(),
            caller,
            destination_chain,
            destination_address,
            payload,
            gas_token,
        );
    }

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        _token_id: String,
        _source_address: Bytes,
        destination_chain: String,
        destination_address: Bytes,
        _amount: i128,
        _metadata: Bytes,
        gas_token: Token,
    ) {
        // TODO: _takeToken, decode metadata, and abi encode with MessageType.InterchainTransfer
        let payload = bytes!(&env,);

        // TODO: Get the params for the cross-chain message, taking routing via ITS Hub into account.

        Self::pay_gas_and_call_contract(
            env.clone(),
            caller,
            destination_chain,
            String::from_val(env, &destination_address.to_val()),
            payload,
            gas_token,
        );
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
}

#[contractimpl]
impl MigratableInterface for InterchainTokenService {
    type MigrationData = ();
    type Error = axelar_gateway::error::ContractError;

    fn migrate(env: &Env, migration_data: ()) -> Result<(), axelar_gateway::error::ContractError> {
        interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| axelar_gateway::error::ContractError::MigrationNotAllowed)
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
