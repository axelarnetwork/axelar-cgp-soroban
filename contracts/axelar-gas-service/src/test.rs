#![cfg(test)]
extern crate std;

use crate::contract::{AxelarGasService, AxelarGasServiceClient};
use soroban_sdk::{
    bytes, bytesn, symbol_short,
    testutils::{Address as _, Events},
    token, vec, Address, BytesN, Env, IntoVal, String, Val, Vec, U256,
};

fn setup_env<'a>() -> (Env, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();

    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGasService);

    let client = AxelarGasServiceClient::new(&env, &contract_id);

    let gas_collector: Address = Address::generate(&env);

    client.initialize(&gas_collector);

    (env, contract_id, client)
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
    let events = env.events().all();
    assert!(event_index < events.len(), "event_index out of bounds");

    let event = events.get(event_index).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}

#[test]
fn pay_native_gas_for_contract_call() {
    let (env, contract_id, client) = setup_env();
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

    // assert_emitted_event(
    //     &env,
    //     0,
    //     &contract_id,
    //     (symbol_short!("cc_g_paid"),),
    //     (
    //         sender,
    //         destination_chain,
    //         destination_address,
    //         env.crypto().keccak256(&payload),
    //     ),
    // );
}

#[test]
fn refund() {
    let (env, contract_id, client) = setup_env();

    let tx_hash: BytesN<32> = bytesn!(
        &env,
        0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d
    );
    let log_index: U256 = U256::from_u32(&env, 1);
    let receiver: Address = Address::generate(&env);
    let token_address: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let supply: i128 = 1000;
    let refund_amount: i128 = 1;

    token::StellarAssetClient::new(&env, &token_address).mint(&contract_id, &supply);
    let token_client = token::Client::new(&env, &token_address);

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

    // assert_emitted_event(
    //     &env,
    //     0,
    //     &contract_id,
    //     (symbol_short!("refunded"), tx_hash, log_index),
    //     (receiver, token_address, amount),
    // );
}
