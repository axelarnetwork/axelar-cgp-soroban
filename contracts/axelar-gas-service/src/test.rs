#![cfg(test)]
extern crate std;

use std::format;

use axelar_soroban_std::{assert_emitted_event, types::Token};

use crate::contract::{AxelarGasService, AxelarGasServiceClient};
use soroban_sdk::{
    bytes, symbol_short,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env, String,
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

    let token_address: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = Token {
        address: token_addr.clone(),
        amount: gas_amount,
    };
    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    let token_client = TokenClient::new(&env, &token_addr);
    StellarAssetClient::new(&env, &token_addr).mint(&sender, &gas_amount);

    let expiration_ledger = &env.ledger().sequence() + 200;

    // approve token spend before invoking `pay_gas_for_contract_call`
    token_client.approve(&sender, &contract_id, &gas_amount, &expiration_ledger);

    assert_eq!(token_client.allowance(&sender, &contract_id), gas_amount);

    client.pay_gas_for_contract_call(
        &sender,
        &destination_chain,
        &destination_address,
        &payload,
        &refund_address,
        &token,
    );

    assert_eq!(0, token_client.balance(&sender));
    assert_eq!(gas_amount, token_client.balance(&contract_id));
    assert_eq!(token_client.allowance(&sender, &contract_id), 0);

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
        (destination_address, payload, refund_address, token),
    );
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, client) = setup_env();
    let token_address: Address = env.register_stellar_asset_contract(Address::generate(&env));
    let token_client = TokenClient::new(&env, &token_addr);
    let supply: i128 = 1000;
    let refund_amount = 1;
    let token = Token {
        address: token_addr.clone(),
        amount: refund_amount,
    };
    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    client.collect_fees(&gas_collector, &token);

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("collected"), gas_collector, token),
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

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: token_addr.clone(),
        amount: refund_amount,
    };

    let message_id = String::from_str(
        &env,
        &format!(
            "{}-{}",
            "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d", 0
        ),
    );

    client.refund(&message_id, &receiver, &token);

    assert_eq!(refund_amount, token_client.balance(&receiver));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("refunded"), message_id, receiver, token),
        (),
    );
}
