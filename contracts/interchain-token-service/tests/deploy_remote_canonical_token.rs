mod utils;

use axelar_soroban_std::{address::AddressExt, events};
use interchain_token_service::{
    event::InterchainTokenDeploymentStartedEvent, types::TokenManagerType,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, String};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::{setup_env, setup_gas_token, TokenMetadataExt};

#[test]
fn deploy_remote_canonical_token_succeeds() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let token_metadata = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_minter = Address::generate(&env);
    let token_id = BytesN::<32>::from_array(&env, &[1; 32]);

    let token_address = env
        .deployer()
        .with_address(sender.clone(), token_id.clone())
        .deploy_v2(
            client.interchain_token_wasm_hash(),
            (
                sender.clone(),
                initial_minter,
                token_id.clone(),
                token_metadata.clone(),
            ),
        );

    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address);
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);

    assert_eq!(client.register_canonical_token(&token_address), expected_id);
    assert_eq!(client.token_address(&expected_id), token_address);

    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    let destination_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let deployed_token_id = client.mock_all_auths().deploy_remote_canonical_token(
        &sender,
        &token_address,
        &destination_chain,
        &gas_token,
    );
    assert_eq!(expected_id, deployed_token_id);

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTokenDeploymentStartedEvent,
    >(&env, -4));
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn deploy_remote_canonical_token_fail_no_actual_token() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let token_address = Address::generate(&env);
    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address);
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);

    assert_eq!(client.register_canonical_token(&token_address), expected_id);
    assert_eq!(client.token_address(&expected_id), token_address);

    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    let destination_chain = String::from_str(&env, "ethereum");
    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().deploy_remote_canonical_token(
        &sender,
        &token_address,
        &destination_chain,
        &gas_token,
    );
}
