use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::AxelarGasServiceClient;
use stellar_axelar_gateway::executable::{AxelarExecutableInterface, CustomAxelarExecutable};
use stellar_axelar_gateway::AxelarGatewayMessagingClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::events::Event;
use stellar_axelar_std::token::StellarAssetClient;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::xdr::ToXdr;
use stellar_axelar_std::{
    contract, contractimpl, ensure, interfaces, only_operator, only_owner, soroban_sdk, vec,
    when_not_paused, Address, AxelarExecutable, Bytes, BytesN, Env, IntoVal, Operatable, Ownable,
    Pausable, String, Symbol, Upgradable, Val,
};
use stellar_token_manager::TokenManagerClient;
use token_id::UnregisteredTokenId;

use crate::error::ContractError;
use crate::event::{
    InterchainTokenDeploymentStartedEvent, InterchainTransferReceivedEvent,
    InterchainTransferSentEvent, LinkTokenReceivedEvent, LinkTokenStartedEvent,
    TokenMetadataRegisteredEvent, TrustedChainRemovedEvent, TrustedChainSetEvent,
};
use crate::flow_limit::FlowDirection;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage::{self, TokenIdConfigValue};
use crate::token_manager::TokenManagerClientExt;
use crate::token_metadata::TokenMetadataExt;
use crate::types::{
    DeployInterchainToken, HubMessage, InterchainTransfer, LinkToken, Message,
    RegisterTokenMetadata, TokenManagerType,
};
use crate::{deployer, flow_limit, token_handler, token_id, token_metadata};

const ITS_HUB_CHAIN_NAME: &str = "axelar";
const EXECUTE_WITH_INTERCHAIN_TOKEN: &str = "execute_with_interchain_token";

#[contract]
#[derive(Operatable, Ownable, Pausable, Upgradable, AxelarExecutable)]
pub struct InterchainTokenService;

#[contractimpl]
impl InterchainTokenService {
    pub fn __constructor(
        env: Env,
        owner: Address,
        operator: Address,
        gateway: Address,
        gas_service: Address,
        its_hub_address: String,
        chain_name: String,
        native_token_address: Address,
        interchain_token_wasm_hash: BytesN<32>,
        token_manager_wasm_hash: BytesN<32>,
    ) {
        interfaces::set_owner(&env, &owner);
        interfaces::set_operator(&env, &operator);
        storage::set_gateway(&env, &gateway);
        storage::set_gas_service(&env, &gas_service);
        storage::set_its_hub_address(&env, &its_hub_address);
        storage::set_chain_name(&env, &chain_name);
        storage::set_native_token_address(&env, &native_token_address);
        storage::set_interchain_token_wasm_hash(&env, &interchain_token_wasm_hash);
        storage::set_token_manager_wasm_hash(&env, &token_manager_wasm_hash);
    }
}

#[contractimpl]
impl InterchainTokenServiceInterface for InterchainTokenService {
    fn gas_service(env: &Env) -> Address {
        storage::gas_service(env)
    }

    fn chain_name(env: &Env) -> String {
        storage::chain_name(env)
    }

    fn its_hub_chain_name(env: &Env) -> String {
        String::from_str(env, ITS_HUB_CHAIN_NAME)
    }

    fn its_hub_address(env: &Env) -> String {
        storage::its_hub_address(env)
    }

    fn native_token_address(env: &Env) -> Address {
        storage::native_token_address(env)
    }

    fn interchain_token_wasm_hash(env: &Env) -> BytesN<32> {
        storage::interchain_token_wasm_hash(env)
    }

    fn token_manager_wasm_hash(env: &Env) -> BytesN<32> {
        storage::token_manager_wasm_hash(env)
    }

    fn is_trusted_chain(env: &Env, chain: String) -> bool {
        storage::is_trusted_chain(env, chain)
    }

    #[only_operator]
    fn set_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        ensure!(
            !storage::is_trusted_chain(env, chain.clone()),
            ContractError::TrustedChainAlreadySet
        );

        storage::set_trusted_chain_status(env, chain.clone());

        TrustedChainSetEvent { chain }.emit(env);

        Ok(())
    }

    #[only_operator]
    fn remove_trusted_chain(env: &Env, chain: String) -> Result<(), ContractError> {
        ensure!(
            storage::is_trusted_chain(env, chain.clone()),
            ContractError::TrustedChainNotSet
        );

        storage::remove_trusted_chain_status(env, chain.clone());

        TrustedChainRemovedEvent { chain }.emit(env);

        Ok(())
    }

    fn interchain_token_id(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32> {
        token_id::interchain_token_id(env, Self::chain_name_hash(env), deployer, salt)
    }

    fn canonical_interchain_token_id(env: &Env, token_address: Address) -> BytesN<32> {
        token_id::canonical_interchain_token_id(env, Self::chain_name_hash(env), token_address)
    }

    fn linked_token_id(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32> {
        token_id::linked_token_id(env, Self::chain_name_hash(env), deployer, salt)
    }

    fn interchain_token_address(env: &Env, token_id: BytesN<32>) -> Address {
        deployer::interchain_token_address(env, token_id)
    }

    fn token_manager_address(env: &Env, token_id: BytesN<32>) -> Address {
        deployer::token_manager_address(env, token_id)
    }

    fn registered_token_address(env: &Env, token_id: BytesN<32>) -> Address {
        storage::token_id_config(env, token_id).token_address
    }

    fn deployed_token_manager(env: &Env, token_id: BytesN<32>) -> Address {
        storage::token_id_config(env, token_id).token_manager
    }

    fn token_manager_type(env: &Env, token_id: BytesN<32>) -> TokenManagerType {
        storage::token_id_config(env, token_id).token_manager_type
    }

    fn flow_limit(env: &Env, token_id: BytesN<32>) -> Option<i128> {
        flow_limit::flow_limit(env, token_id)
    }

    fn flow_out_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_out_amount(env, token_id)
    }

    fn flow_in_amount(env: &Env, token_id: BytesN<32>) -> i128 {
        flow_limit::flow_in_amount(env, token_id)
    }

    #[only_operator]
    fn set_flow_limit(
        env: &Env,
        token_id: BytesN<32>,
        flow_limit: Option<i128>,
    ) -> Result<(), ContractError> {
        flow_limit::set_flow_limit(env, token_id, flow_limit)
    }

    #[when_not_paused]
    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        token_metadata: TokenMetadata,
        initial_supply: i128,
        minter: Option<Address>,
    ) -> Result<BytesN<32>, ContractError> {
        caller.require_auth();

        ensure!(initial_supply >= 0, ContractError::InvalidInitialSupply);
        ensure!(
            initial_supply > 0 || minter.is_some(),
            ContractError::InvalidTokenConfig
        );

        let token_id = Self::interchain_token_id(env, caller.clone(), salt);

        token_metadata.validate()?;

        let unregistered_token_id = token_id::ensure_token_not_registered(env, token_id.clone())?;

        let token_address = Self::deploy_token(env, unregistered_token_id, token_metadata, minter)?;

        if initial_supply > 0 {
            StellarAssetClient::new(env, &token_address).mint(&caller, &initial_supply);
        }

        Ok(token_id)
    }

    #[when_not_paused]
    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        gas_token: Option<Token>,
    ) -> Result<BytesN<32>, ContractError> {
        caller.require_auth();

        let token_id = Self::interchain_token_id(env, caller.clone(), salt);

        Self::deploy_remote_token(env, caller, token_id.clone(), destination_chain, gas_token)?;

        Ok(token_id)
    }

    #[when_not_paused]
    fn register_canonical_token(
        env: &Env,
        token_address: Address,
    ) -> Result<BytesN<32>, ContractError> {
        // Validates the token address and its associated token metadata
        let _ =
            token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        let token_id = Self::canonical_interchain_token_id(env, token_address.clone());

        let unregistered_token_id = token_id::ensure_token_not_registered(env, token_id.clone())?;

        let _: Address = Self::deploy_token_manager(
            env,
            unregistered_token_id,
            token_address,
            TokenManagerType::LockUnlock,
        );

        Ok(token_id)
    }

    #[when_not_paused]
    fn deploy_remote_canonical_token(
        env: &Env,
        token_address: Address,
        destination_chain: String,
        spender: Address,
        gas_token: Option<Token>,
    ) -> Result<BytesN<32>, ContractError> {
        spender.require_auth();

        let token_id = Self::canonical_interchain_token_id(env, token_address);

        Self::deploy_remote_token(env, spender, token_id.clone(), destination_chain, gas_token)?;

        Ok(token_id)
    }

    #[when_not_paused]
    fn register_token_metadata(
        env: &Env,
        token_address: Address,
        spender: Address,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        let token_metadata =
            token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        // RegisterTokenMetadata expects u8 decimals, but TokenMetadata uses u32.
        // The unwrap() is safe because token_metadata validation ensures decimals <= 255.
        let hub_message = HubMessage::RegisterTokenMetadata(RegisterTokenMetadata {
            decimals: u8::try_from(token_metadata.decimal).unwrap(),
            token_address: token_address.to_string_bytes(),
        });

        Self::send_to_hub(env, spender, hub_message, gas_token)?;

        TokenMetadataRegisteredEvent {
            token_address,
            decimals: token_metadata.decimal,
        }
        .emit(env);

        Ok(())
    }

    #[when_not_paused]
    fn register_custom_token(
        env: &Env,
        deployer: Address,
        salt: BytesN<32>,
        token_address: Address,
        token_manager_type: TokenManagerType,
    ) -> Result<BytesN<32>, ContractError> {
        deployer.require_auth();

        // Custom token managers can't be deployed with native interchain token type, which is reserved for interchain tokens
        ensure!(
            token_manager_type != TokenManagerType::NativeInterchainToken,
            ContractError::InvalidTokenManagerType
        );

        // Validates the token address and its associated token metadata
        let _ =
            token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        let token_id = Self::linked_token_id(env, deployer, salt);

        let unregistered_token_id = token_id::ensure_token_not_registered(env, token_id.clone())?;

        let _: Address = Self::deploy_token_manager(
            env,
            unregistered_token_id,
            token_address,
            token_manager_type,
        );

        Ok(token_id)
    }

    #[when_not_paused]
    fn link_token(
        env: &Env,
        deployer: Address,
        salt: BytesN<32>,
        destination_chain: String,
        destination_token_address: Bytes,
        token_manager_type: TokenManagerType,
        link_params: Option<Bytes>,
        gas_token: Option<Token>,
    ) -> Result<BytesN<32>, ContractError> {
        deployer.require_auth();

        ensure!(
            !destination_token_address.is_empty(),
            ContractError::InvalidDestinationTokenAddress
        );

        // Custom token managers can't be deployed with native interchain token type, which is reserved for interchain tokens
        ensure!(
            token_manager_type != TokenManagerType::NativeInterchainToken,
            ContractError::InvalidTokenManagerType
        );

        let token_id = Self::linked_token_id(env, deployer.clone(), salt);
        let token_address = Self::token_id_config(env, token_id.clone())?.token_address;

        let message = Message::LinkToken(LinkToken {
            token_id: token_id.clone(),
            token_manager_type,
            source_token_address: token_address.to_string_bytes(),
            destination_token_address: destination_token_address.clone(),
            params: link_params.clone(),
        });

        LinkTokenStartedEvent {
            token_id: token_id.clone(),
            destination_chain: destination_chain.clone(),
            source_token_address: token_address.to_string_bytes(),
            destination_token_address,
            token_manager_type,
            params: link_params,
        }
        .emit(env);

        Self::pay_gas_and_call_contract(env, deployer, destination_chain, message, gas_token)?;

        Ok(token_id)
    }

    #[when_not_paused]
    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        data: Option<Bytes>,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError> {
        ensure!(amount > 0, ContractError::InvalidAmount);

        ensure!(
            !destination_address.is_empty(),
            ContractError::InvalidDestinationAddress
        );

        if let Some(ref data) = data {
            ensure!(!data.is_empty(), ContractError::InvalidData);
        }

        caller.require_auth();

        token_handler::take_token(
            env,
            &caller,
            Self::token_id_config(env, token_id.clone())?,
            amount,
        )?;

        FlowDirection::Out.add_flow(env, token_id.clone(), amount)?;

        InterchainTransferSentEvent {
            token_id: token_id.clone(),
            source_address: caller.clone(),
            destination_chain: destination_chain.clone(),
            destination_address: destination_address.clone(),
            amount,
            data_hash: data
                .as_ref()
                .map(|data| env.crypto().keccak256(data).into()),
        }
        .emit(env);

        let message = Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: caller.to_string_bytes(),
            destination_address,
            amount,
            data,
        });

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(())
    }

    #[only_owner]
    fn transfer_token_admin(
        env: &Env,
        token_id: BytesN<32>,
        new_admin: Address,
    ) -> Result<(), ContractError> {
        let TokenIdConfigValue {
            token_address,
            token_manager_type,
            token_manager,
        } = Self::token_id_config(env, token_id)?;

        ensure!(
            token_manager_type == TokenManagerType::MintBurn,
            ContractError::InvalidTokenManagerType
        );

        TokenManagerClient::new(env, &token_manager).set_admin(env, &token_address, &new_admin);

        Ok(())
    }
}

impl InterchainTokenService {
    fn send_to_hub(
        env: &Env,
        spender: Address,
        hub_message: HubMessage,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError> {
        let gateway = AxelarGatewayMessagingClient::new(env, &Self::gateway(env));
        let gas_service = AxelarGasServiceClient::new(env, &Self::gas_service(env));

        let hub_chain = Self::its_hub_chain_name(env);
        let hub_address = Self::its_hub_address(env);

        let payload = hub_message.abi_encode(env)?;

        if let Some(gas_token) = gas_token {
            gas_service.pay_gas(
                &env.current_contract_address(),
                &hub_chain,
                &hub_address,
                &payload,
                &spender,
                &gas_token,
                &Bytes::new(env),
            );
        }

        gateway.call_contract(
            &env.current_contract_address(),
            &hub_chain,
            &hub_address,
            &payload,
        );

        Ok(())
    }

    fn pay_gas_and_call_contract(
        env: &Env,
        caller: Address,
        destination_chain: String,
        message: Message,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError> {
        // Validate the destination chain only for non-interchain transfer messages.
        // Self-transfers are allowed only in interchain transfers, as they may be useful
        // when broadcasting to a batch that includes the current chain.
        if !matches!(message, Message::InterchainTransfer(_)) {
            ensure!(
                destination_chain != Self::chain_name(env),
                ContractError::InvalidDestinationChain
            );
        }

        ensure!(
            Self::is_trusted_chain(env, destination_chain.clone()),
            ContractError::UntrustedChain
        );

        let hub_message = HubMessage::SendToHub {
            destination_chain,
            message,
        };

        Self::send_to_hub(env, caller, hub_message, gas_token)?;

        Ok(())
    }

    /// Validate that the message is coming from the ITS Hub and decode the message
    fn get_execute_params(
        env: &Env,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(String, Message), ContractError> {
        ensure!(
            source_chain == Self::its_hub_chain_name(env),
            ContractError::NotHubChain
        );
        ensure!(
            source_address == Self::its_hub_address(env),
            ContractError::NotHubAddress
        );

        let HubMessage::ReceiveFromHub {
            source_chain: original_source_chain,
            message,
        } = HubMessage::abi_decode(env, &payload)?
        else {
            return Err(ContractError::InvalidMessageType);
        };
        ensure!(
            storage::is_trusted_chain(env, original_source_chain.clone()),
            ContractError::UntrustedChain
        );

        Ok((original_source_chain, message))
    }

    fn set_token_id_config(env: &Env, token_id: BytesN<32>, token_data: TokenIdConfigValue) {
        storage::set_token_id_config(env, token_id, &token_data);
    }

    /// Retrieves the configuration value for the specified token ID.
    ///
    /// # Arguments
    /// - `token_id`: A 32-byte unique identifier for the token.
    ///
    /// # Returns
    /// - `Ok(TokenIdConfigValue)`: The configuration value if it exists.
    ///
    /// # Errors
    /// - `ContractError::InvalidTokenId`: If the token ID does not exist in storage.
    fn token_id_config(
        env: &Env,
        token_id: BytesN<32>,
    ) -> Result<TokenIdConfigValue, ContractError> {
        storage::try_token_id_config(env, token_id).ok_or(ContractError::InvalidTokenId)
    }

    fn chain_name_hash(env: &Env) -> BytesN<32> {
        let chain_name = Self::chain_name(env);
        env.crypto().keccak256(&chain_name.to_xdr(env)).into()
    }

    /// Deploys a remote token on a specified destination chain.
    ///
    /// This function retrieves and validates the token's metadata
    /// and emits an event indicating the start of the token deployment process.
    /// It also constructs and sends the deployment message to the remote chain.
    ///
    /// # Arguments
    /// * `caller` - Address of the caller initiating the deployment.
    /// * `token_id` - The token ID for the remote token being deployed.
    /// * `destination_chain` - The name of the destination chain where the token will be deployed.
    /// * `gas_token` - An optional gas token used to pay for gas during the deployment.
    ///
    /// # Errors
    /// - `ContractError::InvalidDestinationChain`: If the `destination_chain` is the current chain.
    /// - `ContractError::InvalidTokenId`: If the token ID is invalid.
    /// - Errors propagated from `token_metadata`.
    /// - Any error propagated from `pay_gas_and_call_contract`.
    ///
    /// # Authorization
    /// - The `caller` must authenticate.
    fn deploy_remote_token(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        gas_token: Option<Token>,
    ) -> Result<(), ContractError> {
        let token_address = Self::token_id_config(env, token_id.clone())?.token_address;
        let TokenMetadata {
            name,
            symbol,
            decimal,
        } = token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        let message = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: token_id.clone(),
            name: name.clone(),
            symbol: symbol.clone(),
            decimals: decimal as u8,
            minter: None,
        });

        InterchainTokenDeploymentStartedEvent {
            token_id,
            token_address,
            destination_chain: destination_chain.clone(),
            name,
            symbol,
            decimals: decimal,
            minter: None,
        }
        .emit(env);

        Self::pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token)?;

        Ok(())
    }

    fn execute_transfer_message(
        env: &Env,
        source_chain: &String,
        message_id: String,
        InterchainTransfer {
            token_id,
            source_address,
            destination_address,
            amount,
            data,
        }: InterchainTransfer,
    ) -> Result<(), ContractError> {
        ensure!(amount > 0, ContractError::InvalidAmount);

        let destination_address = Address::from_string_bytes(&destination_address);

        let token_config_value = Self::token_id_config(env, token_id.clone())?;
        let token_address = token_config_value.token_address.clone();

        FlowDirection::In.add_flow(env, token_id.clone(), amount)?;

        token_handler::give_token(env, &destination_address, token_config_value, amount)?;

        InterchainTransferReceivedEvent {
            source_chain: source_chain.clone(),
            token_id: token_id.clone(),
            source_address: source_address.clone(),
            destination_address: destination_address.clone(),
            amount,
            data_hash: data
                .as_ref()
                .map(|data| env.crypto().keccak256(data).into()),
        }
        .emit(env);

        if let Some(payload) = data {
            Self::execute_contract_with_token(
                env,
                destination_address,
                source_chain,
                message_id,
                source_address,
                payload,
                token_id,
                token_address,
                amount,
            );
        }

        Ok(())
    }

    fn execute_contract_with_token(
        env: &Env,
        destination_address: Address,
        source_chain: &String,
        message_id: String,
        source_address: Bytes,
        payload: Bytes,
        token_id: BytesN<32>,
        token_address: Address,
        amount: i128,
    ) {
        // Due to limitations of the soroban-sdk, there is no type-safe client for contract execution.
        // The invocation might return a value, so we use Val as the return type to avoid panics
        env.invoke_contract::<Val>(
            &destination_address,
            &Symbol::new(env, EXECUTE_WITH_INTERCHAIN_TOKEN),
            vec![
                env,
                source_chain.to_val(),
                message_id.to_val(),
                source_address.to_val(),
                payload.to_val(),
                token_id.to_val(),
                token_address.to_val(),
                amount.into_val(env),
            ],
        );
    }

    fn execute_deploy_message(
        env: &Env,
        DeployInterchainToken {
            token_id,
            name,
            symbol,
            decimals,
            minter,
        }: DeployInterchainToken,
    ) -> Result<(), ContractError> {
        let token_metadata = TokenMetadata::new(name, symbol, decimals as u32)?;

        // Note: attempt to convert a byte string which doesn't represent a valid Soroban address fails at the Host level
        let minter = minter.map(|m| Address::from_string_bytes(&m));

        let unregistered_token_id = token_id::ensure_token_not_registered(env, token_id)?;
        let _: Address = Self::deploy_token(env, unregistered_token_id, token_metadata, minter)?;

        Ok(())
    }

    fn execute_link_token_message(
        env: &Env,
        source_chain: String,
        LinkToken {
            token_id,
            token_manager_type,
            source_token_address,
            destination_token_address,
            params,
        }: LinkToken,
    ) -> Result<(), ContractError> {
        let token_address = Address::from_string_bytes(&destination_token_address);

        // Validates the token address and its associated token metadata
        let _ =
            token_metadata::token_metadata(env, &token_address, &Self::native_token_address(env))?;

        let unregistered_token_id = token_id::ensure_token_not_registered(env, token_id)?;

        let _: Address = Self::deploy_token_manager(
            env,
            unregistered_token_id.clone(),
            token_address,
            token_manager_type,
        );

        LinkTokenReceivedEvent {
            source_chain,
            token_id: unregistered_token_id.into(),
            source_token_address,
            destination_token_address,
            token_manager_type,
            params,
        }
        .emit(env);

        Ok(())
    }

    fn deploy_token_manager(
        env: &Env,
        unregistered_token_id: UnregisteredTokenId,
        token_address: Address,
        token_manager_type: TokenManagerType,
    ) -> Address {
        let token_id: BytesN<32> = unregistered_token_id.into();
        let token_manager = deployer::deploy_token_manager(
            env,
            Self::token_manager_wasm_hash(env),
            token_id.clone(),
            token_address.clone(),
            token_manager_type,
        );

        Self::set_token_id_config(
            env,
            token_id,
            TokenIdConfigValue {
                token_address: token_address.clone(),
                token_manager: token_manager.clone(),
                token_manager_type,
            },
        );

        token_handler::post_token_manager_deploy(
            env,
            token_manager_type,
            token_manager.clone(),
            token_address,
        );

        token_manager
    }

    /// Deploy an interchain token on the current chain and its corresponding token manager.
    ///
    /// # Arguments
    /// * `unregistered_token_id` - The unregistered token ID for the interchain token being deployed.
    /// * `token_metadata` - The metadata for the interchain token being deployed.
    /// * `minter` - An optional address of an additional minter for the interchain token being deployed.
    fn deploy_token(
        env: &Env,
        unregistered_token_id: UnregisteredTokenId,
        token_metadata: TokenMetadata,
        minter: Option<Address>,
    ) -> Result<Address, ContractError> {
        let token_address = deployer::deploy_interchain_token(
            env,
            Self::interchain_token_wasm_hash(env),
            minter,
            unregistered_token_id.clone(),
            token_metadata,
        );

        Self::deploy_token_manager(
            env,
            unregistered_token_id,
            token_address.clone(),
            TokenManagerType::NativeInterchainToken,
        );

        Ok(token_address)
    }
}

impl CustomAxelarExecutable for InterchainTokenService {
    type Error = ContractError;

    fn __gateway(env: &Env) -> Address {
        storage::gateway(env)
    }

    #[when_not_paused]
    fn __execute(
        env: &Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) -> Result<(), Self::Error> {
        let (source_chain, message) =
            Self::get_execute_params(env, source_chain, source_address, payload)?;

        match message {
            Message::InterchainTransfer(message) => {
                Self::execute_transfer_message(env, &source_chain, message_id, message)
            }
            Message::DeployInterchainToken(message) => Self::execute_deploy_message(env, message),
            Message::LinkToken(message) => {
                Self::execute_link_token_message(env, source_chain, message)
            }
        }?;

        Ok(())
    }
}
