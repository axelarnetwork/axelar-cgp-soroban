#![cfg(test)]
extern crate std;

use axelar_soroban_std::{
    assert_invoke_auth_err, assert_invoke_auth_ok, assert_last_emitted_event,
};
use interchain_token::contract::InterchainToken;
use interchain_token::InterchainTokenClient;
use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env, IntoVal, Symbol};

fn setup_env<'a>() -> (Env, InterchainTokenClient<'a>) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(InterchainToken, (&owner,));
    let client = InterchainTokenClient::new(&env, &contract_id);

    (env, client)
}

#[test]
fn register_interchain_token() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let contract_id = env.register(InterchainToken, (&owner,));
    let client = InterchainTokenClient::new(&env, &contract_id);

    assert_eq!(client.owner(), owner);
}

#[test]
fn transfer_ownership() {
    let (env, client) = setup_env();
    let owner = client.owner();
    let new_owner = Address::generate(&env);

    assert_invoke_auth_ok!(owner, client.try_transfer_ownership(&new_owner));

    assert_last_emitted_event(
        &env,
        &client.address,
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
    let (env, client) = setup_env();
    let new_owner = Address::generate(&env);
    assert_invoke_auth_err!(new_owner, client.try_transfer_ownership(&new_owner));
}
