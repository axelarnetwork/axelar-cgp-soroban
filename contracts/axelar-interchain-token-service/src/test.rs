#![cfg(test)]
extern crate std;

use crate::{contract::InterchainTokenService, contract::InterchainTokenServiceClient};
use axelar_soroban_std::assert_contract_err;

use soroban_sdk::{
    testutils::Address as _, Address, Env, String,
};

fn setup_env<'a>() -> (Env, Address, InterchainTokenServiceClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, InterchainTokenService);
    let client = InterchainTokenServiceClient::new(&env, &contract_id);

    (env, contract_id, client)
}

fn initialize_its(
    _env: &Env, 
    client: &InterchainTokenServiceClient,
    owner: Address,
) {
    client.initialize(&owner);
}

#[test]
fn initialize() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    assert_eq!(
        client.owner(), owner
    );
}

// TODO: figure out how to make this work??
// #[test]
// fn fails_if_already_initialized() {
//     let (env, _contract_id, client) = setup_env();
//     let owner = Address::generate(&env);

//     initialize_its(&env, &client, owner.clone());

//     assert_contract_err!(
//         client.try_initialize(&owner),
//         InterchainTokenServiceError::AlreadyInitialized
//     );
// }

#[test]
fn add_trusted_address() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    
    client.set_trusted_address(&chain, &trusted_address);

    assert_eq!(
        client.is_trusted_address(&chain, &trusted_address),
        true
    );
}

#[test]
fn remove_trusted_address() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    
    client.set_trusted_address(&chain, &trusted_address);

    client.remove_trusted_address(&chain, &trusted_address);

    assert_eq!(
        client.is_trusted_address(&chain, &trusted_address),
        false
    );
}

#[test] 
fn transfer_ownership() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let new_owner = Address::generate(&env);

    client.transfer_ownership(&new_owner);

    assert_eq!(
        client.owner(),
        new_owner
    );
}