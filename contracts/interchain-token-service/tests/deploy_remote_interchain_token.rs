mod utils;

use axelar_soroban_std::assert_contract_err;
use axelar_soroban_std::events;
use interchain_token_service::error::ContractError;
use interchain_token_service::event::InterchainTokenDeploymentStartedEvent;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, IntoVal, String};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::setup_env;
use utils::setup_gas_token;

fn setup_token_metadata(env: &Env, name: &str, symbol: &str, decimal: u32) -> TokenMetadata {
    TokenMetadata {
        decimal,
        name: name.into_val(env),
        symbol: symbol.into_val(env),
    }
}

#[test]
fn deploy_remote_interchain_token_succeeds() {
    let (env, client, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 1;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let deployed_token_id = client.mock_all_auths().deploy_remote_interchain_token(
        &sender,
        &salt,
        &destination_chain,
        &gas_token,
    );

    assert_eq!(token_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, -4));
}

#[test]
fn deploy_remote_interchain_token_fails_untrusted_chain() {
    let (env, client, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = setup_token_metadata(&env, "name", "symbol", 6);
    let initial_supply = 1;

    client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");

    assert_contract_err!(
        client.mock_all_auths().try_deploy_remote_interchain_token(
            &sender,
            &salt,
            &destination_chain,
            &gas_token,
        ),
        ContractError::UntrustedChain
    );
}

#[test]
#[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
fn deploy_remote_interchain_token_fails_without_interchain_token() {
    let (env, client, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);

    let destination_chain = String::from_str(&env, "ethereum");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().deploy_remote_interchain_token(
        &sender,
        &salt,
        &destination_chain,
        &gas_token,
    );
}
