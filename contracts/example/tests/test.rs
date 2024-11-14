#![cfg(test)]
extern crate std;

use axelar_gas_service::contract::AxelarGasService;
use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::testutils::{generate_proof, get_approve_hash, initialize, TestSignerSet};
use axelar_gateway::types::Message;
use axelar_gateway::{AxelarGateway, AxelarGatewayClient};
use axelar_soroban_std::assert_last_emitted_event;
use axelar_soroban_std::types::Token;
use example::contract::GmpExample;
use example::GmpExampleClient;
use soroban_sdk::token::{StellarAssetClient, TokenClient};
use soroban_sdk::{
    testutils::Address as _, testutils::BytesN as _, vec, Address, BytesN, Env, String,
};
use soroban_sdk::{Bytes, Symbol};

fn setup_gateway<'a>(env: &Env) -> (AxelarGatewayClient<'a>, Address, TestSignerSet) {
    let gateway_id = env.register_contract(None, AxelarGateway);
    let gateway_client = AxelarGatewayClient::new(env, &gateway_id);
    let owner = Address::generate(env);
    let operator = Address::generate(env);
    let signers = initialize(env, &gateway_client, owner, operator, 0, 5);

    (gateway_client, gateway_id, signers)
}

fn setup_gas_service<'a>(env: &Env) -> (AxelarGasServiceClient<'a>, Address, Address) {
    let gas_service_id = env.register_contract(None, AxelarGasService);
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);
    let gas_collector: Address = Address::generate(&env);

    gas_service_client.initialize_gas_service(&gas_collector);

    (gas_service_client, gas_collector, gas_service_id)
}

fn setup_app<'a>(env: &Env, gateway: &Address, gas_service: &Address) -> GmpExampleClient<'a> {
    let contract_id = env.register_contract(None, GmpExample);
    let client = GmpExampleClient::new(env, &contract_id);

    client.initialize_gmp_example(gateway, gas_service);

    client
}

#[test]
fn test_gmp_example() {
    let env = Env::default();
    env.mock_all_auths();

    let user: Address = Address::generate(&env);

    // Setup source Axelar gateway
    let source_chain = String::from_str(&env, "source");
    let (source_gateway_client, source_gateway_id, _) = setup_gateway(&env);
    let (_source_gas_service_client, _source_gas_collector, source_gas_service_id) =
        setup_gas_service(&env);
    let source_app = setup_app(&env, &source_gateway_id, &source_gas_service_id);

    // Setup destination Axelar gateway
    let destination_chain = String::from_str(&env, "destination");
    let (destination_gateway_client, destination_gateway_id, signers) = setup_gateway(&env);
    let (_destination_gas_service_client, _destination_gas_collector, destination_gas_service_id) =
        setup_gas_service(&env);
    let destination_app = setup_app(&env, &destination_gateway_id, &destination_gas_service_id);

    // Set cross-chain message params
    let source_address = source_app.address.to_string();
    let destination_address = destination_app.address.to_string();
    let payload: Bytes = BytesN::<20>::random(&env).into();
    let payload_hash: BytesN<32> = env.crypto().keccak256(&payload).into();

    // Initiate cross-chain contract call, sending message from source to destination
    let asset = &env.register_stellar_asset_contract_v2(user.clone());
    let gas_amount: i128 = 100;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    let token_client = TokenClient::new(&env, &asset.address());
    StellarAssetClient::new(&env, &asset.address()).mint(&user, &gas_amount);

    let expiration_ledger = &env.ledger().sequence() + 200;

    // approve token spend before invoking `pay_gas_for_contract_call` in `send` function
    token_client.approve(
        &user,
        &source_gas_service_id,
        &gas_amount,
        &expiration_ledger,
    );

    assert_eq!(
        token_client.allowance(&user, &source_gas_service_id),
        gas_amount
    );

    source_app.send(
        &user,
        &destination_chain,
        &destination_address,
        &payload,
        &gas_token,
    );

    // Axelar hub confirms the contract call, i.e Axelar verifiers verify/vote on the emitted event
    let message_id = String::from_str(&env, "test");

    // Confirming message from source Axelar gateway
    assert_last_emitted_event(
        &env,
        &source_gateway_client.address,
        (
            Symbol::new(&env, "contract_called"),
            source_app.address.clone(),
            destination_chain,
            destination_address,
            payload_hash.clone(),
        ),
        payload.clone(),
    );

    // Axelar hub signs the message approval, Signing message approval for destination
    let messages = vec![
        &env,
        Message {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
            source_address: source_address.clone(),
            contract_address: destination_app.address.clone(),
            payload_hash,
        },
    ];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    // Submitting signed message approval to destination Axelar gateway
    destination_gateway_client.approve_messages(&messages, &proof);

    // Executing message on destination app
    destination_app.execute(&source_chain, &message_id, &source_address, &payload);

    assert_last_emitted_event(
        &env,
        &destination_app.address,
        (
            Symbol::new(&env, "executed"),
            source_chain,
            message_id,
            source_address,
        ),
        (payload,),
    );
}
