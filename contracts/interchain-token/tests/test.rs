#![cfg(test)]
extern crate std;

use axelar_soroban_std::{
    assert_contract_err, assert_invoke_auth_err, assert_invoke_auth_ok, assert_last_emitted_event,
};
use interchain_token::contract::InterchainToken;
use interchain_token::error::ContractError;
use interchain_token::InterchainTokenClient;
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

fn setup_env<'a>() -> (Env, Address, InterchainTokenClient<'a>) {
    let env = Env::default();

    let contract_id = env.register_contract(None, InterchainToken);
    let client = InterchainTokenClient::new(&env, &contract_id);

    (env, contract_id, client)
}

fn initialize(_env: &Env, client: &InterchainTokenClient, owner: Address) {
    client.initialize(&owner);
}

#[test]
fn initialize_succeeds() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_eq!(client.owner(), owner);
}

#[test]
fn initialize_fails_if_already_initialized() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_contract_err!(
        client.try_initialize(&owner),
        ContractError::AlreadyInitialized
    );
}

#[test]
fn transfer_ownership() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_eq!(client.owner(), owner);

    assert_invoke_auth_ok!(owner, client.try_transfer_ownership(&new_owner));

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

#[test]
fn transfer_ownership_unauthorized() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    initialize(&env, &client, owner.clone());

    assert_eq!(client.owner(), owner);

    assert_invoke_auth_err!(new_owner, client.try_transfer_ownership(&new_owner));
}
