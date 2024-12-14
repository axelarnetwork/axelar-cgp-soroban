mod utils;

use axelar_soroban_std::{address::AddressExt, assert_contract_err, events};
use interchain_token_service::{
    error::ContractError, event::InterchainTokenIdClaimed, types::TokenManagerType,
};
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, BytesN};
use utils::setup_env;

const PREFIX_CANONICAL_TOKEN_SALT: &str = "canonical-token-salt";

#[test]
fn register_canonical_token_succeeds() {
    let (env, client, _, _) = setup_env();
    let token_address = Address::generate(&env);

    let chain_name = client.chain_name();
    let chain_name_hash: BytesN<32> = env.crypto().keccak256(&(chain_name).to_xdr(&env)).into();
    let expected_deploy_salt = env
        .crypto()
        .keccak256(
            &(
                PREFIX_CANONICAL_TOKEN_SALT,
                chain_name_hash,
                token_address.clone(),
            )
                .to_xdr(&env),
        )
        .into();
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

    assert_contract_err!(
        client.try_register_canonical_token(&token_address),
        ContractError::TokenAlreadyRegistered
    );
}
