use axelar_soroban_std::ensure;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};

use crate::abi::{get_message_type, MessageType as EncodedMessageType};
use crate::error::ContractError;
use crate::event;
use crate::interface::InterchainTokenServiceInterface;
use crate::storage_types::DataKey;
use crate::types::{HubMessage, InterchainTransfer, Message};

use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::AxelarGatewayMessagingClient;

use axelar_gateway::executable::AxelarExecutableInterface;

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
        _token_id: BytesN<32>,
        _destination_chain: String,
        _name: String,
        _symbol: String,
        _decimals: u32,
        _minter: Option<Bytes>,
        _gas_token: Token,
    ) {
        todo!()
    }

    fn deploy_remote_interchain_token(
        _env: &Env,
        _caller: Address,
        _destination_chain: String,
        _token_id: String,
        _gas_token: Token,
    ) {
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
    ) {
        // EXAMPLE implementation only to compile
        let message = Message::InterchainTransfer(InterchainTransfer {
            token_id,
            source_address: Bytes::from_slice(env, &[0]),
            destination_address,
            amount,
            data: metadata,
        });

        let _ = pay_gas_and_call_contract(env, caller, destination_chain, message, gas_token);
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

        let _ = execute_message(
            &env,
            source_chain.clone(),
            message_id.clone(),
            source_address.clone(),
            payload.clone(),
        );

        event::executed(&env, source_chain, message_id, source_address, payload);
    }
}

fn pay_gas_and_call_contract(
    env: &Env,
    caller: Address,
    destination_chain: String,
    message: Message,
    gas_token: Token,
) -> Result<(), ContractError> {
    let gateway = AxelarGatewayMessagingClient::new(env, &InterchainTokenService::gateway(env));
    let gas_service = AxelarGasServiceClient::new(env, &InterchainTokenService::gas_service(env));

    let payload = get_call_params(env, destination_chain, message)?;

    let destination_address = InterchainTokenService::trusted_address(
        env,
        InterchainTokenService::its_hub_chain_name(env),
    )
    .ok_or(ContractError::NoTrustedAddressSet)?;

    gas_service.pay_gas(
        &env.current_contract_address(),
        &InterchainTokenService::its_hub_chain_name(env),
        &destination_address,
        &payload,
        &caller,
        &gas_token,
        &Bytes::new(env),
    );

    gateway.call_contract(
        &env.current_contract_address(),
        &InterchainTokenService::its_hub_chain_name(env),
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

    let (_original_source_chain, message) = get_execute_params(env, source_chain, payload)?;

    match message {
        Message::InterchainTransfer(_) => {
            // TODO
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
    payload: Bytes,
) -> Result<(String, Message), ContractError> {
    let message_type =
        get_message_type(&payload.to_alloc_vec()).map_err(|_| ContractError::InvalidPayload)?;

    match message_type {
        EncodedMessageType::ReceiveFromHub => {
            ensure!(
                source_chain == InterchainTokenService::its_hub_chain_name(env),
                ContractError::UntrustedChain
            );

            let decoded_message =
                HubMessage::abi_decode(env, &payload).map_err(|_| ContractError::InvalidPayload)?;

            let HubMessage::ReceiveFromHub {
                source_chain: original_source_chain,
                message: inner_message,
            } = decoded_message
            else {
                return Err(ContractError::InvalidMessageType);
            };

            let trusted_address =
                InterchainTokenService::trusted_address(env, original_source_chain.clone());
            let routing_identifier = InterchainTokenService::its_hub_routing_identifier(env);

            ensure!(
                trusted_address.is_some_and(|addr| addr == routing_identifier),
                ContractError::UntrustedChain
            );

            Ok((original_source_chain, inner_message))
        }
        _ => Err(ContractError::InvalidMessageType),
    }
}

fn get_call_params(
    env: &Env,
    destination_chain: String,
    message: Message,
) -> Result<Bytes, ContractError> {
    // Note: ITS Hub chain as the actual destination chain for the messsage isn't supported
    ensure!(
        destination_chain != InterchainTokenService::its_hub_chain_name(env),
        ContractError::UntrustedChain
    );

    match InterchainTokenService::trusted_address(env, destination_chain.clone()) {
        Some(destination_address) => {
            ensure!(
                destination_address == InterchainTokenService::its_hub_routing_identifier(env),
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
        _ => Err(ContractError::UntrustedChain),
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::types::*;
    use axelar_soroban_std::assert_ok;
    use soroban_sdk::{testutils::Address as _, Address, Bytes, BytesN, Env, String};
    use std::vec;
    use std::vec::Vec;

    const HUB_CHAIN: &str = "hub_chain";
    const HUB_ADDRESS: &str = "hub_address";

    fn setup_env<'a>() -> (Env, InterchainTokenServiceClient<'a>) {
        let env = Env::default();
        let owner = Address::generate(&env);
        let gateway = Address::generate(&env);
        let gas_service = Address::generate(&env);
        let contract_id = env.register(InterchainTokenService, (&owner, gateway, gas_service));
        let client = InterchainTokenServiceClient::new(&env, &contract_id);

        (env, client)
    }

    fn register_chains(env: &Env, client: &InterchainTokenServiceClient) {
        env.mock_all_auths();

        let chain = String::from_str(&env, HUB_CHAIN);
        client.set_trusted_address(&chain, &client.its_hub_routing_identifier());

        let chain = client.its_hub_chain_name();
        let addr = String::from_str(&env, HUB_ADDRESS);
        client.set_trusted_address(&chain, &addr);
    }

    fn bytes_from_hex(env: &Env, hex_string: &str) -> Bytes {
        let bytes_vec: Vec<u8> = hex::decode(hex_string).unwrap();
        Bytes::from_slice(env, &bytes_vec)
    }

    #[test]
    fn get_call_params_hub_message() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = Message::InterchainTransfer(InterchainTransfer {
            token_id: BytesN::from_array(&env, &[255u8; 32]),
            source_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
            destination_address: bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
            amount: i128::MAX,
            data: Some(bytes_from_hex(&env, "abcd")),
        });

        let expected_payload = assert_ok!(HubMessage::SendToHub {
            destination_chain: String::from_str(&env, HUB_CHAIN),
            message: msg.clone()
        }
        .abi_encode(&env));

        let payload = assert_ok!(env.as_contract(&client.address, || {
            get_call_params(&env, String::from_str(&env, HUB_CHAIN), msg.clone())
        }));

        assert_eq!(payload, expected_payload);
    }

    #[test]
    fn get_call_params_fails_send_directly_to_hub_chain() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: BytesN::from_array(&env, &[1u8; 32]),
            name: String::from_str(&env, &"Test Token"),
            symbol: String::from_str(&env, &"TST"),
            decimals: 18,
            minter: Some(bytes_from_hex(&env, "1234")),
        });

        let destination_chain = client.its_hub_chain_name();
        let result = get_call_params(&env, destination_chain, msg.clone());
        assert!(matches!(result, Err(ContractError::UntrustedChain)));
    }

    #[test]
    fn get_call_params_fails_untrusted_chain() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: BytesN::from_array(&env, &[1u8; 32]),
            name: String::from_str(&env, &"Test Token"),
            symbol: String::from_str(&env, &"TST"),
            decimals: 18,
            minter: Some(bytes_from_hex(&env, "1234")),
        });

        let result = env.as_contract(&client.address, || {
            get_call_params(&env, String::from_str(&env, "untrusted_chain"), msg.clone())
        });
        assert!(matches!(result, Err(ContractError::UntrustedChain)));
    }

    #[test]
    fn get_call_params_fails_invalid_payload() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = Message::DeployInterchainToken(DeployInterchainToken {
            token_id: BytesN::from_array(&env, &[1u8; 32]),
            name: String::from_bytes(&env, &vec![0xF5, 0x90, 0x80]), // invalid UTF-8
            symbol: String::from_str(&env, &"TST"),
            decimals: 18,
            minter: Some(bytes_from_hex(&env, "1234")),
        });

        let result = env.as_contract(&client.address, || {
            get_call_params(&env, String::from_str(&env, HUB_CHAIN), msg.clone())
        });
        assert!(matches!(result, Err(ContractError::InvalidPayload)));
    }

    #[test]
    fn get_execute_params_hub_message() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msgs = vec![
            HubMessage::ReceiveFromHub {
                source_chain: String::from_str(&env, HUB_CHAIN),
                message: Message::InterchainTransfer(InterchainTransfer {
                    token_id: BytesN::from_array(&env, &[255u8; 32]),
                    source_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    destination_address: bytes_from_hex(
                        &env,
                        "4F4495243837681061C4743b74B3eEdf548D56A5",
                    ),
                    amount: i128::MAX,
                    data: Some(bytes_from_hex(&env, "abcd")),
                }),
            },
            HubMessage::ReceiveFromHub {
                source_chain: String::from_str(&env, HUB_CHAIN),
                message: Message::DeployInterchainToken(DeployInterchainToken {
                    token_id: BytesN::from_array(&env, &[1u8; 32]),
                    name: String::from_str(&env, &"Test Token"),
                    symbol: String::from_str(&env, &"TST"),
                    decimals: 18,
                    minter: Some(bytes_from_hex(&env, "1234")),
                }),
            },
        ];

        for msg in msgs {
            let encoded_hub_msg = assert_ok!(msg.clone().abi_encode(&env));
            let (original_chain_name, inner_message) =
                assert_ok!(env.as_contract(&client.address, || {
                    get_execute_params(
                        &env,
                        InterchainTokenService::its_hub_chain_name(&env),
                        encoded_hub_msg,
                    )
                }));

            if let HubMessage::ReceiveFromHub {
                source_chain,
                message,
            } = msg
            {
                assert_eq!(original_chain_name, source_chain);
                assert_eq!(inner_message, message);
            }
        }
    }

    #[test]
    fn get_execute_params_fails_hub_message_sent_from_external_chain() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = HubMessage::ReceiveFromHub {
            source_chain: String::from_str(&env, HUB_CHAIN),
            message: Message::DeployInterchainToken(DeployInterchainToken {
                token_id: BytesN::from_array(&env, &[1u8; 32]),
                name: String::from_str(&env, &"Test Token"),
                symbol: String::from_str(&env, &"TST"),
                decimals: 18,
                minter: Some(bytes_from_hex(&env, "1234")),
            }),
        }
        .abi_encode(&env);

        let result = get_execute_params(&env, String::from_str(&env, "somechain"), msg.unwrap());
        assert!(matches!(result, Err(ContractError::UntrustedChain)));
    }

    #[test]
    fn get_execute_params_fails_hub_message_non_hub_source_chain() {
        let (env, client) = setup_env();
        register_chains(&env, &client);

        let msg = HubMessage::ReceiveFromHub {
            source_chain: String::from_str(&env, "somechain"),
            message: Message::DeployInterchainToken(DeployInterchainToken {
                token_id: BytesN::from_array(&env, &[1u8; 32]),
                name: String::from_str(&env, &"Test Token"),
                symbol: String::from_str(&env, &"TST"),
                decimals: 18,
                minter: Some(bytes_from_hex(&env, "1234")),
            }),
        }
        .abi_encode(&env);

        let result = env.as_contract(&client.address, || {
            get_execute_params(
                &env,
                InterchainTokenService::its_hub_chain_name(&env),
                msg.unwrap(),
            )
        });
        assert!(matches!(result, Err(ContractError::UntrustedChain)));
    }
}
