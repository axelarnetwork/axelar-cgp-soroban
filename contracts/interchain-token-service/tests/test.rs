use axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use axelar_gateway::testutils;
use axelar_gateway::AxelarGatewayClient;
use interchain_token_service::contract::{InterchainTokenService, InterchainTokenServiceClient};
use interchain_token_service::error::ContractError;

use axelar_soroban_std::{assert_contract_err, assert_invoke_auth_err, assert_last_emitted_event};
use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol};

fn setup_gateway<'a>(env: &Env) -> AxelarGatewayClient<'a> {
    let (_, client) = testutils::setup_gateway(env, 0, 5);
    client
}

fn setup_gas_service<'a>(env: &Env) -> AxelarGasServiceClient<'a> {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(&env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    gas_service_client
}

fn setup_env<'a>() -> (Env, InterchainTokenServiceClient<'a>) {
    let env = Env::default();
    let owner = Address::generate(&env);
    let gateway_client = setup_gateway(&env);
    let gas_service_client = setup_gas_service(&env);
    let contract_id = env.register(
        InterchainTokenService,
        (&owner, gateway_client.address, gas_service_client.address),
    );
    let client = InterchainTokenServiceClient::new(&env, &contract_id);

    (env, client)
}

#[test]
fn register_interchain_token_service() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let gateway_client = setup_gateway(&env);
    let gas_service_client = setup_gas_service(&env);
    let contract_id = env.register(
        InterchainTokenService,
        (&owner, gateway_client.address, gas_service_client.address),
    );
    let client = InterchainTokenServiceClient::new(&env, &contract_id);

    assert_eq!(client.owner(), owner);
}

#[test]
fn set_trusted_address() {
    let (env, client) = setup_env();
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
    let (env, client) = setup_env();

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
    let (env, client) = setup_env();
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
    let (env, client) = setup_env();
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
    let (env, client) = setup_env();
    env.mock_all_auths();

    let chain = String::from_str(&env, "chain");

    assert_eq!(client.trusted_address(&chain), None);

    assert_contract_err!(
        client.try_remove_trusted_address(&chain),
        ContractError::NoTrustedAddressSet
    );
}

#[test]
fn transfer_ownership() {
    let (env, client) = setup_env();
    env.mock_all_auths();

    let prev_owner = client.owner();
    let new_owner = Address::generate(&env);

    client.transfer_ownership(&new_owner);

    assert_last_emitted_event(
        &env,
        &client.address,
        (
            Symbol::new(&env, "ownership_transferred"),
            prev_owner,
            new_owner.clone(),
        ),
        (),
    );

    assert_eq!(client.owner(), new_owner);
}
