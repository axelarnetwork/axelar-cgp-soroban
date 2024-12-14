mod utils;

use axelar_soroban_std::{address::AddressExt, assert_contract_err, events};
use interchain_token_service::{
    error::ContractError, event::InterchainTokenIdClaimed, types::TokenManagerType,
};
use soroban_sdk::{testutils::Address as _, Address};
use utils::setup_env;

#[test]
fn register_canonical_token_succeeds() {
    let (env, client, _, _) = setup_env();
    let token_address = Address::generate(&env);

    let expected_deploy_salt = client.canonical_token_deploy_salt(&token_address.clone());
    let expected_id = client.interchain_token_id(&Address::zero(&env), &expected_deploy_salt);

    assert_eq!(client.register_canonical_token(&token_address), expected_id);

    assert_eq!(client.token_address(&expected_id), token_address);

    assert_eq!(
        client.token_manager_type(&expected_id),
        TokenManagerType::LockUnlock
    );

    goldie::assert!(events::fmt_last_emitted_event::<InterchainTokenIdClaimed>(
        &env
    ));
}

#[test]
fn register_canonical_token_fails_if_already_registered() {
    let (env, client, _, _) = setup_env();
    let token_address = Address::generate(&env);

    client.register_canonical_token(&token_address);

    let result = client.try_register_canonical_token(&token_address);

    assert_contract_err!(result, ContractError::TokenAlreadyRegistered);
}
