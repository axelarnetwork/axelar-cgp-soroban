#![cfg(test)]
extern crate std;

use crate::{contract::InterchainTokenService, contract::InterchainTokenServiceClient};

use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke}, Address, Env, IntoVal, String, 
};

fn setup_env<'a>() -> (Env, Address, InterchainTokenServiceClient<'a>) {
    let env = Env::default();

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

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")] // Unauthorized
fn add_trusted_address_fails_if_not_owner() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let not_owner = Address::generate(&env);
    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client.mock_auths(&[MockAuth {
        address: &not_owner,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_trusted_address",
            args: (chain.clone(), trusted_address.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]).set_trusted_address(&chain, &trusted_address);

}

#[test]
fn set_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    
    client.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_trusted_address",
            args: (chain.clone(), trusted_address.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]).set_trusted_address(&chain, &trusted_address);

    assert_eq!(
        client.is_trusted_address(&chain, &trusted_address),
        true
    );
}

#[test]
fn remove_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    
    client.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "set_trusted_address",
            args: (chain.clone(), trusted_address.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]).set_trusted_address(&chain, &trusted_address);

    client.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "remove_trusted_address",
            args: (chain.clone(), trusted_address.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]).remove_trusted_address(&chain, &trusted_address);

    assert_eq!(
        client.is_trusted_address(&chain, &trusted_address),
        false
    );
}

#[test] 
fn transfer_ownership() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let new_owner = Address::generate(&env);

    client.mock_auths(&[MockAuth {
        address: &owner,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "transfer_ownership",
            args: (&new_owner,).into_val(&env),
            sub_invokes: &[],
        },
    }]).transfer_ownership(&new_owner);

    assert_eq!(
        client.owner(),
        new_owner
    );
}

