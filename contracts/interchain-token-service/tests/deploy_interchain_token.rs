use axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use axelar_gateway::testutils;
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::assert_contract_err;
use axelar_soroban_std::assert_invoke_auth_err;
use interchain_token::InterchainTokenClient;
use interchain_token_service::error::ContractError;
use interchain_token_service::{InterchainTokenService, InterchainTokenServiceClient};

use soroban_sdk::BytesN;
use soroban_sdk::IntoVal;
use soroban_sdk::{testutils::Address as _, Address, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;

const INTERCHAIN_TOKEN_WASM_HASH: &[u8] = include_bytes!("testdata/interchain_token.wasm");

fn setup_token_metadata(env: &Env, name: &str, symbol: &str, decimal: u32) -> TokenMetadata {
    TokenMetadata {
        decimal,
        name: name.into_val(env),
        symbol: symbol.into_val(env),
    }
}

fn setup_gateway<'a>(env: &Env) -> AxelarGatewayClient<'a> {
    let (_, client) = testutils::setup_gateway(env, 0, 5);
    client
}

fn setup_gas_service<'a>(env: &Env) -> AxelarGasServiceClient<'a> {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    gas_service_client
}

fn setup_its(env: &Env) -> InterchainTokenServiceClient {
    let owner = Address::generate(&env);
    let gateway_client = setup_gateway(&env);
    let gas_service_client = setup_gas_service(&env);
    let chain_name = String::from_str(&env, "chain_name");

    let wasm_hash = env
        .deployer()
        .upload_contract_wasm(INTERCHAIN_TOKEN_WASM_HASH);

    let contract_id = env.register(
        InterchainTokenService,
        (
            &owner,
            gateway_client.address,
            gas_service_client.address,
            chain_name,
            wasm_hash,
        ),
    );

    InterchainTokenServiceClient::new(&env, &contract_id)
}

#[test]
fn deploy_interchain_token_with_initial_supply_no_minter() {
    let env = Env::default();
    env.mock_all_auths();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let deployed_address = its_client.deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), its_client.address);
    assert!(token.is_minter(&its_client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_with_initial_supply_valid_minter() {
    let env = Env::default();
    env.mock_all_auths();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(Address::generate(&env));
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    let deployed_address = its_client.deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), its_client.address);
    assert!(!token.is_minter(&its_client.address));
    assert!(token.is_minter(&minter.unwrap()));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_zero_initial_supply_and_valid_minter() {
    let env = Env::default();
    env.mock_all_auths();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(Address::generate(&env));
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let deployed_address = its_client.deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), its_client.address);
    assert!(token.is_minter(&its_client.address));
    assert!(!token.is_minter(&sender));
    assert!(token.is_minter(&minter.unwrap()));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_falis_zero_initial_supply_and_invalid_minter() {
    let env = Env::default();
    env.mock_all_auths();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let minter: Option<Address> = Some(its_client.address.clone());
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    assert_contract_err!(
        its_client.try_deploy_interchain_token(
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
    let env = Env::default();
    env.mock_all_auths();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 0;

    let deployed_address = its_client.deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let token = InterchainTokenClient::new(&env, &deployed_address);

    assert_eq!(token.owner(), its_client.address);
    assert!(token.is_minter(&its_client.address));
    assert!(!token.is_minter(&sender));
    assert_eq!(token.balance(&sender), initial_supply);
}

#[test]
fn deploy_interchain_token_fails_with_invalid_auth() {
    let env = Env::default();

    let its_client = setup_its(&env);
    let sender = Address::generate(&env);
    let user = Address::generate(&env);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 100;

    assert_invoke_auth_err!(
        user,
        its_client.try_deploy_interchain_token(
            &sender,
            &salt,
            &token_meta_data,
            &initial_supply,
            &minter,
        )
    );
}
