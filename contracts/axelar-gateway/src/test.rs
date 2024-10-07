#![cfg(test)]
extern crate std;

use axelar_soroban_interfaces::types::Message;
use axelar_soroban_std::{assert_emitted_event, assert_invocation};

use crate::testutils::{
    generate_proof, generate_signer_set, generate_test_message, get_approve_hash,
    get_rotation_hash, initialize, randint,
};
use crate::{contract::AxelarGateway, contract::AxelarGatewayClient};
use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, Events},
    vec, Address, Env, String,
};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
#[should_panic(expected = "Already initialized")]
fn fails_if_already_initialized() {
    let (env, _contract_id, client) = setup_env();
    let operator = Address::generate(&env);

    initialize(&env, &client, operator.clone(), 1, randint(1, 10));

    initialize(&env, &client, operator.clone(), 1, randint(1, 10));
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
        (
            &user,
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
        ),
    );

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (
            symbol_short!("called"),
            user,
            env.crypto().keccak256(&payload),
        ),
        (destination_chain, destination_address, payload),
    );
}

#[test]
fn validate_message() {
    let (env, contract_id, client) = setup_env();

    let (
        Message {
            message_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        },
        _,
    ) = generate_test_message(&env);

    let approved = client.validate_message(
        &contract_address,
        &message_id,
        &source_chain,
        &source_address,
        &payload_hash,
    );
    assert!(!approved);

    assert_invocation(
        &env,
        &contract_address,
        &contract_id,
        "validate_message",
        (
            &contract_address,
            message_id.clone(),
            source_chain.clone(),
            source_address.clone(),
            payload_hash.clone(),
        ),
    );

    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_message() {
    let (env, contract_id, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let Message {
        message_id,
        source_chain,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, operator, 1, randint(1, 10));

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (
            symbol_short!("approved"),
            message_id.clone(),
            contract_address.clone(),
            payload_hash.clone(),
        ),
        (source_chain.clone(), source_address.clone()),
    );

    let is_approved = client.is_message_approved(
        &message_id,
        &source_chain,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(is_approved);

    let approved = client.validate_message(
        &contract_address,
        &message_id,
        &source_chain,
        &source_address,
        &payload_hash,
    );
    assert!(approved);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("executed"), message_id.clone()),
        (),
    );

    let is_approved = client.is_message_approved(
        &message_id,
        &source_chain,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(!is_approved);

    let is_executed = client.is_message_executed(&message_id, &source_chain);
    assert!(is_executed);
}

#[test]
fn fail_execute_invalid_proof() {
    let (env, _contract_id, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, operator, 1, randint(1, 10));

    let invalid_signers = generate_signer_set(&env, randint(1, 10), signers.domain_separator);

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, invalid_signers);

    let res = client.try_approve_messages(&messages, &proof);
    assert!(res.is_err());
}

#[test]
fn approve_messages_skip_duplicate_message() {
    let (env, _, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let operator = Address::generate(&env);

    let signers = initialize(&env, &client, operator, 1, randint(1, 10));

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    // should not throw an error, should just skip
    let res = client.try_approve_messages(&messages, &proof);
    assert!(res.is_ok());

    // should not emit any more events (1 total because of rotate signers)
    assert_eq!(env.events().all().len(), 1);
}

#[test]
fn rotate_signers() {
    let (env, contract_id, client) = setup_env();
    let operator = Address::generate(&env);

    let signers = initialize(&env, &client, operator, 1, randint(1, 10));

    let new_signers = generate_signer_set(&env, randint(1, 10), signers.domain_separator.clone());

    let data_hash = get_rotation_hash(&env, new_signers.signer_set.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.rotate_signers(&new_signers.signer_set, &proof);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("rotated"),),
        (new_signers.signer_set.clone(),),
    );

    // test approve with new signer set
    let (message, _) = generate_test_message(&env);
    let Message {
        message_id,
        source_chain,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, new_signers);
    client.approve_messages(&messages, &proof);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (
            symbol_short!("approved"),
            message_id.clone(),
            contract_address,
            payload_hash,
        ),
        (source_chain, source_address),
    );
}
