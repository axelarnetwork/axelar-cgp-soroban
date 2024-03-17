#![cfg(test)]
extern crate std;

use axelar_soroban_std::{assert_emitted_event, types::TokenDetails};

use crate::contract::{AxelarGasService, AxelarGasServiceClient};
use soroban_sdk::{
    bytes, bytesn, symbol_short,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env, String, U256,
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

#[test]
fn pay_gas_for_contract_call() {
    let (env, contract_id, _, client) = setup_env();

    let token_addr: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token_details = TokenDetails {
        token_addr: token_addr.clone(),
        amount: gas_amount,
    };
    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    let token_client = TokenClient::new(&env, &token_addr);
    StellarAssetClient::new(&env, &token_addr).mint(&sender, &gas_amount);

    client.pay_gas_for_contract_call(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &refund_address,
        &token_details,
    );

    assert_eq!(0, token_client.balance(&sender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (
            symbol_short!("gas_paid"),
            env.crypto().keccak256(&payload),
            sender,
            destination_chain,
        ),
        (destination_address, payload, refund_address, token_details),
    );
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, client) = setup_env();
    let token_addr: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = TokenClient::new(&env, &token_addr);
    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &token_addr).mint(&contract_id, &supply);

    let refund_amount = 1;

    client.collect_fees(&gas_collector, &token_addr, &refund_amount);

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (
            symbol_short!("collected"),
            gas_collector,
            token_addr,
            refund_amount,
        ),
        (),
    );
}

#[test]
fn refund() {
    let (env, contract_id, _, client) = setup_env();
    let token_addr: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = TokenClient::new(&env, &token_addr);
    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &token_addr).mint(&contract_id, &supply);

    let tx_hash = bytesn!(
        &env,
        0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d
    );
    let log_index: U256 = U256::from_u32(&env, 0);
    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;

    client.refund(&tx_hash, &log_index, &receiver, &token_addr, &refund_amount);

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("refunded"), tx_hash, log_index, receiver),
        (token_addr, refund_amount),
    );
}
