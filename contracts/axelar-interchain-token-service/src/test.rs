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

fn initialize_its(_env: &Env, client: &InterchainTokenServiceClient, owner: Address) {
    client.initialize_its(&owner);
}

#[test]
fn initialize() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    assert_eq!(client.owner(), owner);
}

#[test]
fn initialize_fails_if_already_initialized() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    assert_contract_err!(
        client.try_initialize_its(&owner),
        ContractError::AlreadyInitialized
    );
}

#[test]
fn set_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &trusted_address);

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

    initialize_its(&env, &client, owner.clone());

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
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    let another_trusted_address = String::from_str(&env, "another_trusted_address");

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &trusted_address);

    assert_contract_err!(
        client
            .mock_auths(&[MockAuth {
                address: &owner,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "set_trusted_address",
                    args: (chain.clone(), trusted_address.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }])
            .try_set_trusted_address(&chain, &trusted_address),
        ContractError::TrustedAddressAlreadySet
    );

    // to change trusted address, owner must remove and set again
    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "remove_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_trusted_address(&chain, &trusted_address);

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), another_trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &another_trusted_address);
}

#[test]
fn remove_trusted_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &trusted_address);

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "remove_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .remove_trusted_address(&chain, &trusted_address);

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
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    assert_eq!(client.trusted_address(&chain), None);

    assert_contract_err!(
        client
            .mock_auths(&[MockAuth {
                address: &owner,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "remove_trusted_address",
                    args: (chain.clone(), trusted_address.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }])
            .try_remove_trusted_address(&chain, &trusted_address),
        ContractError::NotTrustedAddress
    );
}

#[test]
fn remove_trusted_address_fails_for_incorrect_address() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "set_trusted_address",
                args: (chain.clone(), trusted_address.clone()).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .set_trusted_address(&chain, &trusted_address);

    let not_trusted_address = String::from_str(&env, "not_trusted_address");

    assert_contract_err!(
        client
            .mock_auths(&[MockAuth {
                address: &owner,
                invoke: &MockAuthInvoke {
                    contract: &contract_id,
                    fn_name: "remove_trusted_address",
                    args: (chain.clone(), not_trusted_address.clone()).into_val(&env),
                    sub_invokes: &[],
                },
            }])
            .try_remove_trusted_address(&chain, &not_trusted_address),
        ContractError::NotTrustedAddress
    );
}

#[test]
fn transfer_ownership() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);

    initialize_its(&env, &client, owner.clone());

    let new_owner = Address::generate(&env);

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_ownership",
                args: (&new_owner,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_ownership(&new_owner);

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
