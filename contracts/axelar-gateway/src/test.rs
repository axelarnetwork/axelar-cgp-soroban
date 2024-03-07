#![cfg(test)]
extern crate std;

use axelar_soroban_std::{assert_emitted_event, assert_invocation};

use axelar_auth_verifier::testutils::{generate_proof, randint};

use crate::testutils::{generate_test_approval, initialize};
use crate::types::{self, SignedCommandBatch};
use crate::{contract::AxelarGateway, AxelarGatewayClient};
use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, BytesN as _, Events},
    vec,
    xdr::ToXdr,
    Address, BytesN, Env, String,
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
fn validate_contract_call() {
    let (env, contract_id, client) = setup_env();

    let (
        types::ContractCallApproval {
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        },
        _,
    ) = generate_test_approval(&env);

    let command_id = BytesN::random(&env);

    let approved = client.validate_contract_call(
        &contract_address,
        &command_id,
        &source_chain,
        &source_address,
        &payload_hash,
    );
    assert!(!approved);

    assert_invocation(
        &env,
        &contract_address,
        &contract_id,
        "validate_contract_call",
        (
            &contract_address,
            command_id.clone(),
            source_chain.clone(),
            source_address.clone(),
            payload_hash.clone(),
        ),
    );

    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_contract_call() {
    let (env, contract_id, client) = setup_env();
    let (approval, _) = generate_test_approval(&env);
    let types::ContractCallApproval {
        source_chain,
        source_address,
        contract_address,
        payload_hash,
    } = approval.clone();
    let command_id = BytesN::random(&env);

    let signers = initialize(&env, &client, 1, randint(1, 10));

    let batch = types::CommandBatch {
        chain_id: 1,
        commands: vec![
            &env,
            (
                command_id.clone(),
                types::Command::ContractCallApproval(approval),
            ),
        ],
    };
    let batch_hash = env.crypto().keccak256(&batch.clone().to_xdr(&env));

    let signed_batch = SignedCommandBatch {
        batch,
        proof: generate_proof(&env, batch_hash, signers).to_xdr(&env),
    };

    client.execute(&signed_batch.to_xdr(&env));

    assert_emitted_event(
        &env,
        -2,
        &contract_id,
        (
            symbol_short!("approved"),
            command_id.clone(),
            contract_address,
            payload_hash,
        ),
        (source_chain, source_address),
    );

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("command"), command_id),
        (),
    );
}
