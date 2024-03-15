#![cfg(any(test, feature = "testutils"))]
extern crate std;

use crate::{types, contract::AxelarGatewayClient};
use axelar_auth_verifier::{contract::AxelarAuthVerifier, testutils::TestSignerSet};
use rand::Rng;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

pub fn initialize(
    env: &Env,
    client: &AxelarGatewayClient,
    previous_signer_retention: u32,
    num_signers: u32,
) -> TestSignerSet {
    let auth_contract_id = env.register_contract(None, AxelarAuthVerifier);

    let auth_client =
        axelar_auth_verifier::contract::AxelarAuthVerifierClient::new(env, &auth_contract_id);

    let signers = axelar_auth_verifier::testutils::initialize(
        env,
        &auth_client,
        client.address.clone(),
        previous_signer_retention,
        num_signers,
    );

    client.initialize(&auth_contract_id);

    signers
}

pub fn generate_test_approval(env: &Env) -> (types::ContractCallApproval, Bytes) {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(0..20);
    let mut payload = std::vec![0u8; len];
    rng.fill(&mut payload[..]);

    let payload = Bytes::from_slice(env, &payload[..]);

    (
        types::ContractCallApproval {
            source_chain: String::from_str(env, DESTINATION_CHAIN),
            source_address: String::from_str(env, DESTINATION_ADDRESS),
            contract_address: Address::generate(env),
            payload_hash: env.crypto().keccak256(&payload),
        },
        payload,
    )
}
