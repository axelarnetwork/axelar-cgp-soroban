#![cfg(test)]
extern crate std;

use axelar_auth_verifier::contract::AxelarAuthVerifier;

use axelar_auth_verifier::testutils::{generate_proof, TestSignerSet};

use axelar_gateway::contract::{AxelarGateway, AxelarGatewayClient};
use axelar_gateway::testutils::get_approve_hash;
use axelar_soroban_interfaces::types::Message;
use axelar_soroban_std::assert_emitted_event;
use soroban_sdk::{contract, contractimpl, log, symbol_short, Bytes};
use soroban_sdk::{testutils::BytesN as _, vec, Address, BytesN, Env, String};

use axelar_soroban_interfaces::axelar_executable::AxelarExecutableInterface;

#[contract]
pub struct AxelarApp;

#[contractimpl]
impl AxelarExecutableInterface for AxelarApp {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&"gateway").unwrap()
    }

    fn execute(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) {
        Self::validate(
            env.clone(),
            message_id,
            source_chain,
            source_address,
            payload.clone(),
        );

        env.events()
            .publish((symbol_short!("executed"),), (payload,));
    }
}

#[contractimpl]
impl AxelarApp {
    pub fn initialize(env: Env, gateway: Address) {
        env.storage().instance().set(&"initialized", &true);

        env.storage().instance().set(&"gateway", &gateway);
    }

    pub fn send(env: Env, destination_chain: String, destination_address: String, message: Bytes) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }
}

fn setup_gateway<'a>(env: &Env) -> (AxelarGatewayClient<'a>, TestSignerSet) {
    let gateway_id = env.register_contract(None, AxelarGateway);
    let gateway_client = AxelarGatewayClient::new(env, &gateway_id);

    let auth_contract_id = env.register_contract(None, AxelarAuthVerifier);
    let auth_client =
        axelar_auth_verifier::contract::AxelarAuthVerifierClient::new(env, &auth_contract_id);

    let signers = axelar_auth_verifier::testutils::initialize(
        env,
        &auth_client,
        gateway_client.address.clone(),
        0,
        5,
    );

    gateway_client.initialize(&auth_contract_id, &gateway_client.address.clone());

    (gateway_client, signers)
}

fn setup_app<'a>(env: &Env, gateway: &Address) -> AxelarAppClient<'a> {
    let contract_id = env.register_contract(None, AxelarApp);
    let client = AxelarAppClient::new(env, &contract_id);
    client.initialize(gateway);

    client
}

#[test]
/// This is an integration test for sending a cross-chain message from source to destination Axelar powered app
fn test_gmp() {
    let env = Env::default(); // Auth doesn't need to be mocked for this integration test

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, "source");
    let (source_gateway_client, _) = setup_gateway(&env);
    let source_app = setup_app(&env, &source_gateway_client.address);

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, "destination");
    let (destination_gateway_client, signers) = setup_gateway(&env);
    let destination_gateway_id = destination_gateway_client.address.clone();
    let destination_app = setup_app(&env, &destination_gateway_id);
    let destination_app_id = destination_app.address.clone();

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app_id.to_string();
    let payload: Bytes = BytesN::<20>::random(&env).into();
    let payload_hash = env.crypto().keccak256(&payload);

    // Initiate cross-chain contract call
    log!(env, "Sending message from source to destination");

    source_app.send(&destination_chain, &destination_address, &payload);

    // Axelar hub confirms the contract call, i.e Axelar verifiers verify/vote on the emitted event
    let message_id = String::from_str(&env, "test");

    log!(env, "Confirming message from source Axelar gateway");

    assert_emitted_event(
        &env,
        -1,
        &source_gateway_client.address,
        (
            symbol_short!("called"),
            source_app.address.clone(),
            payload_hash.clone(),
        ),
        (destination_chain, destination_address, payload.clone()),
    );

    // Axelar hub signs the message approval
    log!(env, "Signing message approval for destination");

    let messages = vec![
        &env,
        Message {
            message_id: message_id.clone(),
            source_chain: source_chain.clone(),
            source_address: source_address.clone(),
            contract_address: destination_app_id.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    // Submit the signed batch to the destination Axelar gateway
    log!(
        env,
        "Submitting signed message approval to destination Axelar gateway"
    );

    destination_gateway_client.approve_messages(&messages, &proof);

    // Execute the app
    log!(env, "Executing message on destination app");

    destination_app.execute(&message_id, &source_chain, &source_address, &payload);

    assert_emitted_event(
        &env,
        -1,
        &destination_app_id,
        (symbol_short!("executed"),),
        (payload,),
    );
}
