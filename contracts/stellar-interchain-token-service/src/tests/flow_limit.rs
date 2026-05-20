use std::println;

use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_gateway::testutils::{approve_gateway_messages, TestSignerSet};
use stellar_axelar_gateway::types::Message as GatewayMessage;
use stellar_axelar_gateway::AxelarGatewayClient;
use stellar_axelar_std::address::AddressExt;
use stellar_axelar_std::testutils::{Address as _, Ledger as _};
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::{
    assert_auth, assert_contract_err, assert_ok, events, vec, Address, Bytes, BytesN, Env, String,
};
use stellar_interchain_token::InterchainTokenClient;

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::FlowLimitSetEvent;
use crate::testutils::setup_its_token;
use crate::types::{HubMessage, InterchainTransfer, Message};
use crate::InterchainTokenServiceClient;

struct GatewayConfig<'a> {
    client: AxelarGatewayClient<'a>,
    signers: TestSignerSet,
}

struct TokenConfig {
    id: BytesN<32>,
    deployer: Address,
}

struct ApprovedMessage {
    source_chain: String,
    message_id: String,
    source_address: String,
    payload: Bytes,
}

const EPOCH_TIME: u64 = 6 * 60 * 60;

const fn dummy_flow_limit() -> i128 {
    1000
}

fn dummy_transfer_params(env: &Env) -> (String, Bytes, Option<Bytes>) {
    let destination_chain = String::from_str(env, "ethereum");
    let destination_address = Bytes::from_hex(env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = None;

    (destination_chain, destination_address, data)
}

fn setup<'a>() -> (
    Env,
    InterchainTokenServiceClient<'a>,
    GatewayConfig<'a>,
    TokenConfig,
) {
    let (env, client, gateway_client, _, signers) = setup_env();

    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let supply = 0;
    let deployer = Address::generate(&env);
    let (token_id, _) = setup_its_token(&env, &client, &deployer, supply);

    client.mock_all_auths().set_flow_limit(
        &client.operator(),
        &token_id,
        &Some(dummy_flow_limit()),
    );

    (
        env,
        client,
        GatewayConfig {
            client: gateway_client,
            signers,
        },
        TokenConfig {
            id: token_id,
            deployer,
        },
    )
}

fn approve_its_transfer(
    env: &Env,
    client: &InterchainTokenServiceClient,
    gateway: &GatewayConfig,
    token_id: &BytesN<32>,
    amount: i128,
) -> ApprovedMessage {
    let sender = Address::generate(env).to_string_bytes();
    let recipient = Address::generate(env).to_string_bytes();
    let source_chain = client.its_hub_chain_name();
    let source_address = client.its_hub_address();

    let msg = HubMessage::ReceiveFromHub {
        source_chain: source_chain.clone(),
        message: Message::InterchainTransfer(InterchainTransfer {
            token_id: token_id.clone(),
            source_address: sender,
            destination_address: recipient,
            amount,
            data: None,
        }),
    };
    let payload = msg.abi_encode(env).unwrap();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    let message_id = Address::generate(env).to_string();

    let messages = vec![
        &env,
        GatewayMessage {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: client.address.clone(),
            payload_hash,
        },
    ];

    approve_gateway_messages(env, &gateway.client, gateway.signers.clone(), messages);

    ApprovedMessage {
        source_chain,
        message_id,
        source_address,
        payload,
    }
}

fn execute_its_transfer(
    env: &Env,
    client: &InterchainTokenServiceClient,
    gateway: &GatewayConfig,
    token_id: &BytesN<32>,
    amount: i128,
) {
    let msg = approve_its_transfer(env, client, gateway, token_id, amount);

    client.execute(
        &msg.source_chain,
        &msg.message_id,
        &msg.source_address,
        &msg.payload,
    )
}

#[derive(Debug, Clone)]
enum Flow {
    In(i128),
    Out(i128),
}

impl Flow {
    const fn flip(self) -> Self {
        match self {
            Self::In(amount) => Self::Out(amount),
            Self::Out(amount) => Self::In(amount),
        }
    }
}

#[derive(Debug, Clone)]
struct TestCase {
    flow_limit: i128,
    flows: std::vec::Vec<Flow>,
}

impl TestCase {
    /// Flips the direction of the flows in the test case. The test case should still logically behave in the same way.
    fn flip(self) -> Self {
        Self {
            flow_limit: self.flow_limit,
            flows: self.flows.into_iter().map(Flow::flip).collect(),
        }
    }
}

fn execute_test_case(
    env: &Env,
    client: &InterchainTokenServiceClient,
    gateway: &GatewayConfig,
    token: &TokenConfig,
    test_case: TestCase,
    expected_error: Option<ContractError>,
) {
    println!("Executing test case: {:?}", test_case);

    client.mock_all_auths().set_flow_limit(
        &client.operator(),
        &token.id,
        &Some(test_case.flow_limit),
    );

    let (destination_chain, destination_address, data) = dummy_transfer_params(env);
    let token_address = client.registered_token_address(&token.id);

    if !client.is_trusted_chain(&destination_chain) {
        client
            .mock_all_auths()
            .set_trusted_chain(&destination_chain);
    }

    #[allow(clippy::manual_try_fold)]
    let result = test_case
        .flows
        .into_iter()
        .fold(Ok(Ok(())), |previous_result, flow| {
            assert_ok!(assert_ok!(previous_result));

            println!("Executing flow: {:?}", flow);

            match flow {
                Flow::In(amount) => {
                    let msg = approve_its_transfer(env, client, gateway, &token.id, amount);

                    client.try_execute(
                        &msg.source_chain,
                        &msg.message_id,
                        &msg.source_address,
                        &msg.payload,
                    )
                }
                Flow::Out(amount) => {
                    let gas_token = setup_gas_token(env, &token.deployer);

                    let token_client = InterchainTokenClient::new(env, &token_address);
                    token_client.mock_all_auths().mint(&token.deployer, &amount);

                    client.mock_all_auths().try_interchain_transfer(
                        &token.deployer,
                        &token.id,
                        &destination_chain,
                        &destination_address,
                        &amount,
                        &data,
                        &Some(gas_token),
                    )
                }
            }
        });

    match expected_error {
        Some(error) => assert_contract_err!(result, error),
        None => assert_ok!(assert_ok!(result)),
    }
}

fn execute_test_cases(test_cases: std::vec::Vec<TestCase>, expected_error: Option<ContractError>) {
    // Flipped test cases should produce the same expected error
    let all_test_cases = test_cases
        .clone()
        .into_iter()
        .chain(test_cases.into_iter().map(|test_case| test_case.flip()));

    for test_case in all_test_cases {
        let (env, client, gateway, token) = setup();

        execute_test_case(&env, &client, &gateway, &token, test_case, expected_error);
    }
}

#[test]
fn set_flow_limit_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    assert_eq!(client.flow_limit(&token_id), None);

    let operator = client.operator();

    assert_auth!(
        &operator,
        client.set_flow_limit(&operator, &token_id, &Some(dummy_flow_limit()))
    );
    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));

    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit()));
}

#[test]
fn set_flow_limit_to_none_succeeds() {
    let (env, client, _, token) = setup();

    assert_eq!(client.flow_limit(&token.id), Some(dummy_flow_limit()));

    let operator = client.operator();

    assert_auth!(
        &operator,
        client.set_flow_limit(&operator, &token.id, &None::<i128>)
    );
    goldie::assert!(events::fmt_last_emitted_event::<FlowLimitSetEvent>(&env));

    assert_eq!(client.flow_limit(&token.id), None);
}

#[test]
fn set_flow_limit_fails_on_negative_limit() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);

    let invalid_limit = Some(-1);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_set_flow_limit(&client.operator(), &token_id, &invalid_limit),
        ContractError::InvalidFlowLimit
    );
}

#[test]
fn set_flow_limit_fails_if_caller_not_authorized() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let stranger = Address::generate(&env);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_set_flow_limit(&stranger, &token_id, &Some(dummy_flow_limit())),
        ContractError::NotApprovedFlowLimiter
    );
}

#[test]
fn flow_limit_resets_after_epoch() {
    let (env, client, gateway, token) = setup();

    let amount = dummy_flow_limit();

    execute_its_transfer(&env, &client, &gateway, &token.id, amount);

    assert_eq!(client.flow_in_amount(&token.id), amount);

    env.ledger()
        .set_timestamp(env.ledger().timestamp() + EPOCH_TIME);

    assert_eq!(client.flow_in_amount(&token.id), 0);
}

#[test]
fn add_flow_succeeds() {
    let test_cases = std::vec![
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(1)],
        },
        TestCase {
            flow_limit: 1000,
            flows: std::vec![Flow::In(1000)],
        },
        TestCase {
            flow_limit: i128::MAX,
            flows: std::vec![Flow::In(i128::MAX)],
        },
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(1), Flow::Out(1), Flow::In(1)],
        },
        TestCase {
            flow_limit: 10,
            flows: std::vec![
                Flow::In(5),
                Flow::In(5),
                Flow::Out(10),
                Flow::Out(10),
                Flow::In(1),
                Flow::In(10),
                Flow::In(9)
            ],
        },
        TestCase {
            flow_limit: i128::MAX,
            flows: std::vec![Flow::In(1), Flow::Out(1), Flow::In(1)],
        },
        TestCase {
            flow_limit: i128::MAX,
            flows: std::vec![Flow::In(i128::MAX - 1), Flow::Out(i128::MAX), Flow::In(1)],
        },
    ];

    execute_test_cases(test_cases, None);
}

#[test]
fn zero_flow_limit_effectively_freezes_token() {
    let test_cases = std::vec![
        TestCase {
            flow_limit: 0,
            flows: std::vec![Flow::In(1)],
        },
        TestCase {
            flow_limit: 0,
            flows: std::vec![Flow::In(i128::MAX)],
        },
    ];

    execute_test_cases(test_cases, Some(ContractError::FlowAmountExceededLimit));
}

#[test]
fn add_flow_fails_on_flow_amount_exceeded_limit() {
    let test_cases = std::vec![
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(2)],
        },
        TestCase {
            flow_limit: i128::MAX - 1,
            flows: std::vec![Flow::In(i128::MAX)],
        },
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(1), Flow::Out(2)],
        },
        TestCase {
            flow_limit: i128::MAX - 1,
            flows: std::vec![Flow::In(1), Flow::Out(i128::MAX)],
        },
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(1), Flow::Out(1), Flow::Out(1), Flow::In(2)],
        },
    ];

    execute_test_cases(test_cases, Some(ContractError::FlowAmountExceededLimit));
}

#[test]
fn add_flow_fails_on_flow_amount_overflow() {
    let test_cases = std::vec![
        TestCase {
            flow_limit: i128::MAX,
            flows: std::vec![Flow::In(i128::MAX), Flow::In(1)],
        },
        TestCase {
            flow_limit: i128::MAX - 1,
            flows: std::vec![Flow::In(1), Flow::Out(2), Flow::Out(i128::MAX - 1)],
        },
        TestCase {
            flow_limit: i128::MAX - 1,
            flows: std::vec![Flow::In(10), Flow::Out(1), Flow::In(i128::MAX - 2)],
        },
        TestCase {
            flow_limit: i128::MAX / 2 + 2,
            flows: std::vec![
                Flow::In(i128::MAX / 2),
                Flow::Out(i128::MAX / 2),
                Flow::In((i128::MAX / 2) + 2),
            ],
        },
    ];

    execute_test_cases(test_cases, Some(ContractError::FlowAmountOverflow));
}

#[test]
fn add_flow_fails_on_flow_limit_exceeded() {
    let test_cases = std::vec![
        TestCase {
            flow_limit: 1,
            flows: std::vec![Flow::In(1), Flow::In(1)],
        },
        TestCase {
            flow_limit: 10,
            flows: std::vec![Flow::In(10), Flow::Out(10), Flow::In(1), Flow::In(10)],
        },
        TestCase {
            flow_limit: i128::MAX - 1,
            flows: std::vec![Flow::In(i128::MAX - 1), Flow::In(1)],
        },
        TestCase {
            flow_limit: i128::MAX - 2,
            flows: std::vec![Flow::In(1), Flow::Out(i128::MAX - 2), Flow::Out(2)],
        },
        TestCase {
            flow_limit: i128::MAX / 2,
            flows: std::vec![Flow::In(i128::MAX / 2), Flow::Out(1), Flow::In(2)],
        },
    ];

    execute_test_cases(test_cases, Some(ContractError::FlowLimitExceeded));
}
