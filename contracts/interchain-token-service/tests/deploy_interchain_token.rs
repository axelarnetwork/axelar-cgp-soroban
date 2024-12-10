mod utils;

use axelar_soroban_std::address::AddressExt;
use axelar_soroban_std::assert_contract_err;
use axelar_soroban_std::assert_invoke_auth_err;
use interchain_token::InterchainTokenClient;
use interchain_token_service::error::ContractError;

use soroban_sdk::Address;
use soroban_sdk::BytesN;
use soroban_sdk::IntoVal;
use soroban_sdk::{testutils::Address as _, Env};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::setup_env;

fn setup_token_metadata(env: &Env, name: &str, symbol: &str, decimal: u32) -> TokenMetadata {
    TokenMetadata {
        decimal,
        name: name.into_val(env),
        symbol: symbol.into_val(env),
    }
}

#[test]
fn deploy_interchain_token_with_initial_supply_no_minter() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let (deployed_address, _token_id) =
        client.deploy_interchain_token(&sender, &salt, &token_meta_data, &initial_supply, &minter);

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_with_initial_supply_valid_minter() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(Address::generate(&env));
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let (deployed_address, _token_id) =
        client.deploy_interchain_token(&sender, &salt, &token_meta_data, &initial_supply, &minter);

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), client.address);
    assert!(!token.is_minter(&client.address));
    assert!(token.is_minter(&minter.unwrap()));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_check_token_id() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(Address::generate(&env));
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let deploy_salt = client.interchain_token_deploy_salt(&sender, &salt);
    let expected_token_id = client.interchain_token_id(&Address::zero(&env), &deploy_salt);

    let (_deployed_address, token_id) =
        client.deploy_interchain_token(&sender, &salt, &token_meta_data, &initial_supply, &minter);

    assert_eq!(token_id, expected_token_id);
}

#[test]
fn deploy_interchain_token_zero_initial_supply_and_valid_minter() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(Address::generate(&env));
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let (deployed_address, _token_id) =
        client.deploy_interchain_token(&sender, &salt, &token_meta_data, &initial_supply, &minter);

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert!(token.is_minter(&minter.unwrap()));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_falis_zero_initial_supply_and_invalid_minter() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(client.address.clone());
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    assert_contract_err!(
        client.try_deploy_interchain_token(
            &sender,
            &salt,
            &token_meta_data,
            &initial_supply,
            &minter
        ),
        ContractError::InvalidMinter
    );
}

#[test]
fn deploy_interchain_token_zero_initial_supply_no_minter() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let (deployed_address, _token_id) =
        client.deploy_interchain_token(&sender, &salt, &token_meta_data, &initial_supply, &minter);

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), client.address);
    assert!(token.is_minter(&client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_fails_with_invalid_auth() {
    let (env, client, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let user = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    assert_invoke_auth_err!(
        user,
        client.try_deploy_interchain_token(
            &sender,
            &salt,
            &token_meta_data,
            &initial_supply,
            &minter,
        )
    );
}
