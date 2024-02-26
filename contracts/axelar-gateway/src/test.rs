#![cfg(test)]
extern crate std;

// use axelar_auth_verifier::contract::{AxelarAuthVerifier, AxelarAuthVerifierClient};

mod axelar_auth_verifier {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/axelar_auth_verifier.wasm"
    );
}

use crate::{contract::AxelarGateway, AxelarGatewayClient};
use crate::types;
use soroban_sdk::{bytes, symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events}, vec, xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Val, Vec};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let auth_contract_id = env.register_contract_wasm(None, axelar_auth_verifier::WASM);

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    client.initialize(&auth_contract_id);

    (env, contract_id, client)
}

fn generate_test_approval(env: &Env) -> (types::ContractCallApproval, Bytes) {
    let payload = bytes!(&env, 0x1234);

    (types::ContractCallApproval {
        source_chain: String::from_str(env, DESTINATION_CHAIN),
        source_address: String::from_str(env, DESTINATION_ADDRESS),
        contract_address: Address::generate(env),
        payload_hash: env.crypto().keccak256(&payload),
    }, payload)
}

fn assert_invocation(env: &Env, caller: &Address, contract_id: &Address, function_name: &str, args: Vec<Val>) {
    assert_eq!(
        env.auths(),
        std::vec![
            (
            caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(env, function_name),
                    args,
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
fn assert_emitted_event(env: &Env, event_index: u32, contract_id: &Address, topics: Vec<Val>, data: Val) {
    let events = env.events().all();

    assert!(event_index < events.len(), "event_index out of bounds");
    let event = events.get(event_index).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics);
    assert_eq!(vec![env, event.2], vec![env, data]);
}

#[test]
fn call_contract() {
    let (env, contract_id, client) = setup_env();

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    client.call_contract(&user, &destination_chain, &destination_address, &payload);

    assert_invocation(
        &env,
        &user,
        &contract_id,
        "call_contract",
        (&user, destination_chain.clone(), destination_address.clone(), payload.clone()).into_val(&env),
    );

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (
            symbol_short!("called"),
            user,
            env.crypto().keccak256(&payload),
        ).into_val(&env),
        (destination_chain, destination_address, payload).into_val(&env),
    );
}

#[test]
fn validate_contract_call() {
    let (env, contract_id, client) = setup_env();

    let (types::ContractCallApproval {
        source_chain,
        source_address,
        contract_address,
        payload_hash,
    }, _) = generate_test_approval(&env);

    let command_id = BytesN::random(&env);

    let approved = client.validate_contract_call(&contract_address, &command_id, &source_chain, &source_address, &payload_hash);
    assert!(!approved);

    assert_invocation(
        &env,
        &contract_address,
        &contract_id,
        "validate_contract_call",
        (&contract_address, command_id.clone(), source_chain.clone(), source_address.clone(), payload_hash.clone()).into_val(&env),
    );

    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_contract_call() {
    let (env, contract_id, client) = setup_env();
    let (approval , _) = generate_test_approval(&env);
    let types::ContractCallApproval { source_chain, source_address, contract_address, payload_hash } = approval.clone();
    let command_id = BytesN::random(&env);

    let signed_batch = types::SignedCommandBatch {
        batch: types::CommandBatch {
            chain_id: 1,
            commands: vec![
                &env,
                (command_id.clone(), types::Command::ContractCallApproval(approval)),
            ],
        },
        proof: Bytes::new(&env),
    };

    client.execute(&signed_batch.to_xdr(&env));

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (
            symbol_short!("approved"),
            command_id.clone(),
            contract_address,
            payload_hash,
        ).into_val(&env),
        (source_chain, source_address).into_val(&env),
    );

    assert_emitted_event(
        &env,
        1,
        &contract_id,
        (
            symbol_short!("command"),
            command_id,
        ).into_val(&env),
        ().into_val(&env),
    );
}
