mod utils;

use axelar_soroban_std::{address::AddressExt, assert_contract_err, events};
use interchain_token_service::{
    error::ContractError, event::InterchainTokenIdClaimedEvent, types::TokenManagerType,
};
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, BytesN};
use utils::setup_env;

const PREFIX_CANONICAL_TOKEN_SALT: &str = "canonical-token-salt";

#[test]
fn register_canonical_token_succeeds() {
    let (env, client, _, _, _) = setup_env();
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

    goldie::assert!(events::fmt_last_emitted_event::<
        InterchainTokenIdClaimedEvent,
    >(&env));
}

#[test]
fn register_canonical_token_fails_if_already_registered() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    client.register_canonical_token(&token_address);

    assert_contract_err!(
        client.try_register_canonical_token(&token_address),
        ContractError::TokenAlreadyRegistered
    );
}

#[test]
fn canonical_token_id_derivation() {
    let (env, client, _, _, _) = setup_env();
    let token_address = Address::generate(&env);

    let chain_name = client.chain_name();
    let chain_name_hash: BytesN<32> = env.crypto().keccak256(&(chain_name).to_xdr(&env)).into();

    let deploy_salt = env
        .crypto()
        .keccak256(
            &(
                PREFIX_CANONICAL_TOKEN_SALT,
                chain_name_hash.clone(),
                token_address,
            )
                .to_xdr(&env),
        )
        .into();

    let token_id = client.interchain_token_id(&Address::zero(&env), &deploy_salt);

    goldie::assert_json!(vec![
        hex::encode(chain_name_hash.to_array()),
        hex::encode(deploy_salt.to_array()),
        hex::encode(token_id.to_array())
    ]);
}
