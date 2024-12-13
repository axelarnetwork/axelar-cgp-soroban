mod utils;

use axelar_gateway::testutils::{generate_proof, get_approve_hash};
use axelar_gateway::types::Message as GatewayMessage;
use soroban_sdk::{testutils::Address as _, vec, Address, Bytes, BytesN, String};
use utils::setup_env;

#[test]
#[should_panic(expected = "Error(Contract, #1)")] // ExecutableError::NotApproved
fn execute_fails_without_gateway_approval() {
    let (env, client, _, _) = setup_env();

    let source_chain = String::from_str(&env, "chain");
    let message_id = String::from_str(&env, "test");
    let source_address = String::from_str(&env, "source");
    let payload = Bytes::new(&env);

    client.execute(&source_chain, &message_id, &source_address, &payload);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")] // ContractError::InsufficientMessageLength
fn test_execute_fails_with_invalid_message() {
    let (env, client, gateway_client, signers) = setup_env();

    let source_chain = client.its_hub_chain_name();
    let message_id = String::from_str(&env, "test");
    let source_address = Address::generate(&env).to_string();

    let invalid_payload = Bytes::from_array(&env, &[1u8; 16]);
    let payload_hash: BytesN<32> = env.crypto().keccak256(&invalid_payload).into();

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
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    gateway_client.approve_messages(&messages, &proof);

    client.execute(
        &source_chain,
        &message_id,
        &source_address,
        &invalid_payload,
    );
}
