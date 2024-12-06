use axelar_soroban_std::{assert_contract_err, assert_invoke_auth_err, assert_last_emitted_event};
use interchain_token_service::error::ContractError;
use interchain_token_service::testutils::setup_env;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, String, Symbol};

#[test]
fn set_trusted_address() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client.set_trusted_address(&chain, &trusted_address);

    assert_last_emitted_event(
        &env,
        &client.address,
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
fn set_trusted_address_fails_if_not_owner() {
    let (env, client, _, _) = setup_env();

    let not_owner = Address::generate(&env);
    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    assert_invoke_auth_err!(
        not_owner,
        client.try_set_trusted_address(&chain, &trusted_address)
    );
}

#[test]
fn set_trusted_address_fails_if_already_set() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");
    let another_trusted_address = String::from_str(&env, "another_trusted_address");

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
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let chain = String::from_str(&env, "chain");
    let trusted_address = String::from_str(&env, "trusted_address");

    client.set_trusted_address(&chain, &trusted_address);

    client.remove_trusted_address(&chain);

    assert_last_emitted_event(
        &env,
        &client.address,
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
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let chain = String::from_str(&env, "chain");

    assert_eq!(client.trusted_address(&chain), None);

    assert_contract_err!(
        client.try_remove_trusted_address(&chain),
        ContractError::NoTrustedAddressSet
    );
}
