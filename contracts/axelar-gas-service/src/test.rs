#![cfg(test)]
extern crate std;

use std::format;

use crate::error::ContractError;
use axelar_soroban_std::{
    assert_contract_err, assert_last_emitted_event, assert_some, types::Token,
};
use crate::{
    contract::{AxelarGasService, AxelarGasServiceClient},
    storage_types::DataKey,
};
use soroban_sdk::{
    bytes,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    Address, Env, String, Symbol,
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
fn fail_not_initialized() {
    // only collect_fees() and refund() require initialization, so setup and call those.

    // do setup_env without initializing at the end
    let env = Env::default();

    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGasService);

    let client = AxelarGasServiceClient::new(&env, &contract_id);
    let gas_collector: Address = Address::generate(&env);

    // collect_fees() setup

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );

    let supply: i128 = 1000;
    let refund_amount = 1;
    let token = Token {
        address: asset.address,
        amount: refund_amount,
    };
    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_contract_err!(
        client.try_collect_fees(&gas_collector, &token),
        ContractError::NotInitialized
    );

    // refund() setup

    let receiver: Address = Address::generate(&env);
    let message_id = String::from_str(
        &env,
        &format!(
            "{}-{}",
            "0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d", 0
        ),
    );

    assert_contract_err!(
        client.try_refund(&message_id, &receiver, &token),
        ContractError::NotInitialized
    );
}

#[test]
fn test_initialize() {
    let (env, contract_id, gas_collector, _client) = setup_env();

    assert!(assert_some!(env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
    })));

    let stored_collector_address = assert_some!(env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<DataKey, Address>(&DataKey::GasCollector)
    }));
    assert_eq!(stored_collector_address, gas_collector);
}

#[test]
fn fail_already_initialized() {
    let (_env, _contract_id, gas_collector, client) = setup_env();

    assert_contract_err!(
        client.try_initialize(&gas_collector),
        ContractError::AlreadyInitialized
    );
}

#[test]
fn fail_pay_gas_zero_gas_amount() {
    let (env, contract_id, _gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );

    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 0;
    let token = Token {
        address: asset.address.clone(),
        amount: gas_amount,
    };
    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    let token_client = TokenClient::new(&env, &asset.address.clone());
    StellarAssetClient::new(&env, &asset.address).mint(&sender, &gas_amount);

    let expiration_ledger = &env.ledger().sequence() + 200;

    // approve token spend before invoking `pay_gas_for_contract_call`
    token_client.approve(&sender, &contract_id, &gas_amount, &expiration_ledger);

    assert_eq!(token_client.allowance(&sender, &contract_id), gas_amount);

    assert_contract_err!(
        client.try_pay_gas_for_contract_call(
            &sender,
            &destination_chain,
            &destination_address,
            &payload,
            &refund_address,
            &token,
        ),
        ContractError::InvalidAmount
    );
}

#[test]
fn pay_gas_for_contract_call() {
    let (env, contract_id, _gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );
    let sender: Address = Address::generate(&env);
    let gas_amount: i128 = 1;
    let token = Token {
        address: asset.address.clone(),
        amount: gas_amount,
    };

    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    let destination_chain: String = String::from_str(&env, "ethereum");
    let destination_address: String =
        String::from_str(&env, "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59");

    let token_client = TokenClient::new(&env, &asset.address);
    StellarAssetClient::new(&env, &asset.address).mint(&sender, &gas_amount);

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

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_paid"),
            env.crypto().keccak256(&payload),
            sender,
            destination_chain,
        ),
        (destination_address, payload, refund_address, token),
    );
}

#[test]
fn fail_collect_fees_zero_refund_amount() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );

    let supply: i128 = 1000;
    let refund_amount = 0;

    let token = Token {
        address: asset.address,
        amount: refund_amount,
    };
    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_contract_err!(
        client.try_collect_fees(&gas_collector, &token),
        ContractError::InvalidAmount
    );
}

#[test]
fn fail_collect_fees_insufficient_balance() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );

    let supply: i128 = 5;
    let refund_amount = 10;

    let token = Token {
        address: asset.address,
        amount: refund_amount,
    };
    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    assert_contract_err!(
        client.try_collect_fees(&gas_collector, &token),
        ContractError::InsufficientBalance
    );
}

#[test]
fn collect_fees() {
    let (env, contract_id, gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );
    let token_client = TokenClient::new(&env, &asset.address);
    let supply: i128 = 1000;
    let refund_amount = 1;
    let token = Token {
        address: asset.address,
        amount: refund_amount,
    };
    StellarAssetClient::new(&env, &token.address).mint(&contract_id, &supply);

    client.collect_fees(&gas_collector, &token);

    assert_eq!(refund_amount, token_client.balance(&gas_collector));
    assert_eq!(supply - refund_amount, token_client.balance(&contract_id));

    assert_last_emitted_event(
        &env,
        &contract_id,
        (Symbol::new(&env, "gas_collected"), gas_collector, token),
        (),
    );
}

#[test]
fn refund() {
    let (env, contract_id, _gas_collector, client) = setup_env();

    let asset = StellarAssetClient::new(
        &env,
        &env.register_stellar_asset_contract(Address::generate(&env)),
    );
    let token_client = TokenClient::new(&env, &asset.address);
    let supply: i128 = 1000;
    StellarAssetClient::new(&env, &asset.address).mint(&contract_id, &supply);

    let receiver: Address = Address::generate(&env);
    let refund_amount: i128 = 1;
    let token = Token {
        address: asset.address,
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

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "gas_refunded"),
            message_id,
            receiver,
            token,
        ),
        (),
    );
}
