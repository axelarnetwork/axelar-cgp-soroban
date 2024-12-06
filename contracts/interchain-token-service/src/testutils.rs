#![cfg(feature = "testutils")]
extern crate std;

use crate::{InterchainTokenService, InterchainTokenServiceClient};

use alloy_primitives::hex;
use axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use axelar_gateway::testutils::{self, TestSignerSet};
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::types::Token;
use soroban_sdk::{
    testutils::Address as _, token::StellarAssetClient, Address, Bytes, Env, String,
};
use std::vec::Vec;

const HUB_CHAIN: &str = "hub_chain";
const HUB_ADDRESS: &str = "hub_address";

pub fn setup_gateway<'a>(env: &Env) -> (TestSignerSet, AxelarGatewayClient<'a>) {
    let (signers, client) = testutils::setup_gateway(env, 0, 5);
    (signers, client)
}

pub fn setup_gas_service<'a>(env: &Env) -> AxelarGasServiceClient<'a> {
    let gas_collector: Address = Address::generate(&env);
    let gas_service_id = env.register(AxelarGasService, (&gas_collector,));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    gas_service_client
}

pub fn setup_env<'a>() -> (
    Env,
    InterchainTokenServiceClient<'a>,
    AxelarGatewayClient<'a>,
    TestSignerSet,
) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let (signers, gateway_client) = setup_gateway(&env);
    let gas_service_client = setup_gas_service(&env);
    let contract_id = env.register(
        InterchainTokenService,
        (&owner, &gateway_client.address, gas_service_client.address),
    );
    let client = InterchainTokenServiceClient::new(&env, &contract_id);

    (env, client, gateway_client, signers)
}

pub fn setup_gas_token(env: &Env, sender: &Address) -> Token {
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(&env));
    let gas_amount: i128 = 1;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    StellarAssetClient::new(&env, &asset.address()).mint(&sender, &gas_amount);

    gas_token
}

pub fn register_chains(env: &Env, client: &InterchainTokenServiceClient) {
    env.mock_all_auths();

    let chain = String::from_str(&env, HUB_CHAIN);
    client.set_trusted_address(&chain, &client.its_hub_routing_identifier());

    let chain = client.its_hub_chain_name();
    let addr = String::from_str(&env, HUB_ADDRESS);
    client.set_trusted_address(&chain, &addr);
}

pub fn bytes_from_hex(env: &Env, hex_string: &str) -> Bytes {
    let bytes_vec: Vec<u8> = hex::decode(hex_string).unwrap();
    Bytes::from_slice(env, &bytes_vec)
}
