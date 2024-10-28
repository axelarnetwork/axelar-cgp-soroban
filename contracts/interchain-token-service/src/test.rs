#![cfg(test)]
extern crate std;

use crate::error::ContractError;
use crate::{contract::InterchainTokenService, contract::InterchainTokenServiceClient};

use axelar_soroban_std::{assert_contract_err, assert_last_emitted_event};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    Address, Env, IntoVal, String, Symbol,
};

fn setup_env<'a>() -> (Env, Address, InterchainTokenServiceClient<'a>) {
    let env = Env::default();

    let contract_id = env.register_contract(None, InterchainTokenService);
    let client = InterchainTokenServiceClient::new(&env, &contract_id);

    (env, contract_id, client)
}

fn initialize(_env: &Env, client: &InterchainTokenServiceClient, owner: Address) {
    client.initialize_its(&owner);
}

#[test]
fn initialize_succeeds() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_eq!(client.owner(), owner);
}

#[test]
fn initialize_fails_if_already_initialized() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_contract_err!(
        client.try_initialize_its(&owner),
        ContractError::AlreadyInitialized
    );
}

#[test]
fn set_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner);

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    env.mock_all_auths();

    client.set_trusted_address(&chain, &trusted_address);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "trusted_address_set"),
            chain.clone(),
            trusted_address.clone(),
        ),
        (),
    );

    assert_eq!(client.trusted_address(&chain), Some(trusted_address));
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
fn set_trusted_address_fails_if_not_owner() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner);

    let not_owner = Address::generate(&env);
    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client
        .mock_auths(&[MockAuth {
            address: &not_owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &trusted_address);
}

#[test]
fn set_trusted_address_fails_if_already_set() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner);

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    let another_trusted_address = String::from_str(&env, "another_trusted_address");

    env.mock_all_auths();

    client.set_trusted_address(&chain, &trusted_address);

    assert_contract_err!(
        client.try_set_trusted_address(&chain, &trusted_address),
        ContractError::TrustedAddressAlreadySet
    );

    client.remove_trusted_address(&chain);

    client.set_trusted_address(&chain, &another_trusted_address);
}

#[test]
fn remove_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner);

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    env.mock_all_auths();

    client.set_trusted_address(&chain, &trusted_address);

    client.remove_trusted_address(&chain);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "trusted_address_removed"),
            chain.clone(),
            trusted_address,
        ),
        (),
    );

    assert_eq!(client.trusted_address(&chain), None);
}

#[test]
fn remove_trusted_address_fails_if_address_not_set() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner);

    let chain = String::from_str(&env, "chain");

    assert_eq!(client.trusted_address(&chain), None);

    env.mock_all_auths();

    assert_contract_err!(
        client.try_remove_trusted_address(&chain),
        ContractError::NoTrustedAddressSet
    );
}

#[test]
fn transfer_ownership() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    let new_owner = Address::generate(&env);

    env.mock_all_auths();

    client.transfer_ownership(&new_owner);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "ownership_transferred"),
            owner,
            new_owner.clone(),
        ),
        (),
    );

    assert_eq!(client.owner(), new_owner);
}
