#![cfg(any(test, feature = "testutils"))]
extern crate std;

use crate::{contract::AxelarGatewayClient, types::CommandType};
use axelar_auth_verifier::{contract::AxelarAuthVerifier, testutils::TestSignerSet};
use axelar_soroban_interfaces::types::{Message, WeightedSigners};
use rand::Rng;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::BytesN;
use soroban_sdk::{testutils::Address as _, Address, Bytes, Env, String, Vec};

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

    client.initialize(&auth_contract_id, &client.address.clone());

    signers
}

pub fn get_approve_hash(env: &Env, messages: Vec<Message>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(CommandType::ApproveMessages, messages).to_xdr(env)).into()
}

pub fn get_rotation_hash(env: &Env, new_signers: WeightedSigners) -> BytesN<32> {
    env.crypto()
        .keccak256(&(CommandType::RotateSigners, new_signers).to_xdr(env)).into()
}

pub fn generate_test_message(env: &Env) -> (Message, Bytes) {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(0..20);
    let mut payload = std::vec![0u8; len];
    rng.fill(&mut payload[..]);

    let payload = Bytes::from_slice(env, &payload[..]);

    (
        Message {
            message_id: String::from_str(env, "test"),
            source_chain: String::from_str(env, DESTINATION_CHAIN),
            source_address: String::from_str(env, DESTINATION_ADDRESS),
            contract_address: Address::generate(env),
            payload_hash: env.crypto().keccak256(&payload).into(),
        },
        payload,
    )
}
