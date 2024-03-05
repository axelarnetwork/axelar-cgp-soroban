#![cfg(test)]
extern crate std;

use axelar_auth_verifier::contract::AxelarAuthVerifier;

use axelar_auth_verifier::testutils::generate_proof;

use axelar_gateway::contract::{AxelarGateway, AxelarGatewayClient};
use axelar_gateway::testutils::generate_test_approval;
use axelar_gateway::types::{self, CommandBatch, ContractCallApproval, SignedCommandBatch};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    vec,
    xdr::ToXdr,
    Address, BytesN, Env, String,
};

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
fn test_gmp() {
    let (env, _, _) = setup_env();

    // Setup source gateway
    let source_gateway_id = env.register_contract(None, AxelarGateway);
    let source_gateway_client = AxelarGatewayClient::new(&env, &source_gateway_id);

    // Setup destination gateway
    let destination_gateway_id = env.register_contract(None, AxelarGateway);
    let destination_gateway_client = AxelarGatewayClient::new(&env, &destination_gateway_id);

    let auth_contract_id = env.register_contract(None, AxelarAuthVerifier);
    let auth_client =
        axelar_auth_verifier::contract::AxelarAuthVerifierClient::new(&env, &auth_contract_id);

    let signers = axelar_auth_verifier::testutils::initialize(
        &env,
        &auth_client,
        source_gateway_client.address.clone(),
        0,
        5,
    );

    destination_gateway_client.initialize_gateway(&auth_contract_id);

    let (
        ContractCallApproval {
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        },
        payload,
    ) = generate_test_approval(&env);

    // Initiate cross-chain contract call
    source_gateway_client.call_contract(
        &Address::generate(&env),
        &String::from_str(&env, "destination"),
        &contract_address.to_string(),
        &payload,
    );

    // Axelar hub confirms the message

    // Axelar hub signs the message approval
    let command_id = BytesN::random(&env);
    let batch = CommandBatch {
        chain_id: 1,
        commands: vec![
            &env,
            (
                command_id.clone(),
                types::Command::ContractCallApproval(ContractCallApproval {
                    source_chain: source_chain.clone(),
                    source_address: source_address.clone(),
                    contract_address: contract_address.clone(),
                    payload_hash: payload_hash.clone(),
                }),
            ),
        ],
    };
    let batch_hash = env.crypto().keccak256(&batch.clone().to_xdr(&env));

    let proof = generate_proof(&env, batch_hash, signers);

    let signed_batch = SignedCommandBatch {
        batch,
        proof: proof.to_xdr(&env),
    };

    // Submit the signed batch
    destination_gateway_client.execute(&signed_batch.to_xdr(&env));

    // Validate the contract call
    destination_gateway_client.validate_contract_call(
        &contract_address,
        &command_id,
        &source_chain,
        &source_address,
        &payload_hash,
    );
}
