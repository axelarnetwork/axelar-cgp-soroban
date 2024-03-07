#![cfg(test)]
extern crate std;

use crate::contract::{AxelarGasService, AxelarGasServiceClient};
use soroban_sdk::{
    bytes, bytesn, symbol_short,
    testutils::{Address as _, Events},
    token::{StellarAssetClient, TokenClient},
    vec, Address, BytesN, Env, IntoVal, String, Val, Vec, U256,
};

fn setup_env<'a>() -> (Env, Address, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();

    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGasService);

    let client = AxelarGasServiceClient::new(&env, &contract_id);
    let gas_collector: Address = Address::generate(&env);
    client.initialize(&gas_collector);

    (env, contract_id, gas_collector, client)
}

fn setup_env_with_token<'a>() -> (
    Env,
    Address,
    Address,
    Address,
    TokenClient<'a>,
    i128,
    AxelarGasServiceClient<'a>,
) {
    let (env, contract_id, gas_collector, client) = setup_env();
    let token_address: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = TokenClient::new(&env, &token_address);
    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &token_address).mint(&contract_id, &supply);

    (
        env,
        contract_id,
        gas_collector,
        token_address,
        token_client,
        supply,
        client,
    )
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
fn assert_emitted_event<U, V>(
    env: &Env,
    event_index: u32,
    contract_id: &Address,
    topics: U,
    data: V,
) where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    let event = env.events().all().get(event_index).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}

#[test]
fn pay_native_gas_for_contract_call() {
    let (env, contract_id, _, client) = setup_env();
    let sender: Address = Address::generate(&env);
    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    client.pay_native_gas_for_contract_call(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &refund_address,
    );

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (symbol_short!("cc_g_paid"), env.crypto().keccak256(&payload)),
        (
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
        ),
    );
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, token_address, token_client, supply, client) =
        setup_env_with_token();

    let refund_amount = 1;

    assert_eq!(0, token_client.balance(&gas_collector));
    assert_eq!(supply, token_client.balance(&contract_id));

    client.collect_fees(&gas_collector, &token_address, &refund_amount);

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));
    
    assert_emitted_event(
        &env,
        3, //events 0-2 are related to token setup and transfer
        &contract_id,
        (symbol_short!("coll_fees"),),
        (gas_collector, token_address, refund_amount),
    );
}

#[test]
fn refund() {
    let (env, contract_id, _, token_address, token_client, supply, client) = setup_env_with_token();

    let tx_hash: BytesN<32> = bytesn!(
        &env,
        0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d
    );
    let log_index: U256 = U256::from_u32(&env, 0);
    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;

    assert_eq!(0, token_client.balance(&receiver));
    assert_eq!(supply, token_client.balance(&contract_id));

    client.refund(
        &tx_hash,
        &log_index,
        &receiver,
        &token_address,
        &refund_amount,
    );

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        2, //events 0 and 1 are related to token setup
        &contract_id,
        (symbol_short!("refunded"), tx_hash, log_index),
        (receiver, token_address, refund_amount),
    );
}
