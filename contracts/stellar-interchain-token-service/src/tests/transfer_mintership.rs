use stellar_axelar_std::testutils::Address as _;
use stellar_axelar_std::{assert_auth, assert_auth_err, assert_contract_err, Address, BytesN};
use stellar_interchain_token::InterchainTokenClient;

use super::utils::setup_env;
use crate::error::ContractError;
use crate::testutils::setup_its_token;
use crate::types::TokenManagerType;

#[test]
fn transfer_mintership_succeeds() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let minter = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let (token_id, _) = setup_its_token(&env, &client, &minter, 1000);
    let token = InterchainTokenClient::new(&env, &client.registered_token_address(&token_id));

    assert!(token.is_minter(&minter));
    assert!(!token.is_minter(&new_minter));

    assert_auth!(
        minter,
        client.transfer_mintership(&token_id, &minter, &new_minter)
    );

    assert!(!token.is_minter(&minter));
    assert!(token.is_minter(&new_minter));
}

#[test]
fn transfer_mintership_fails_if_caller_not_minter() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let minter = Address::generate(&env);
    let new_minter = Address::generate(&env);
    let stranger = Address::generate(&env);

    let (token_id, _) = setup_its_token(&env, &client, &minter, 1000);

    // The stranger authorizes the call, but does not hold the minter role.
    assert_auth_err!(
        stranger,
        client.transfer_mintership(&token_id, &minter, &new_minter)
    );
}

#[test]
fn transfer_mintership_fails_if_minter_does_not_hold_role() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let minter = Address::generate(&env);
    let stranger = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let (token_id, _) = setup_its_token(&env, &client, &minter, 1000);

    // `stranger` authorizes as itself, so authorization passes. Without an explicit check here
    // this would let any address grant the minter role away, because the deployed token version
    // does not reject removing a role that is not held.
    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_mintership(&token_id, &stranger, &new_minter),
        ContractError::NotMinter
    );
}

#[test]
fn transfer_mintership_fails_if_new_minter_already_minter() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let minter = Address::generate(&env);

    let (token_id, _) = setup_its_token(&env, &client, &minter, 1000);
    let token_manager = client.deployed_token_manager(&token_id);

    // The token manager is added as a minter at deployment, so it already holds the role.
    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_mintership(&token_id, &minter, &token_manager),
        ContractError::MinterAlreadyExists
    );
}

#[test]
fn transfer_mintership_fails_with_non_native_token_manager_types() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let deployer = Address::generate(&env);
    let new_minter = Address::generate(&env);

    for token_manager_type in [
        TokenManagerType::MintBurnFrom,
        TokenManagerType::LockUnlock,
        TokenManagerType::MintBurn,
    ]
    .into_iter()
    {
        let salt = BytesN::<32>::from_array(&env, &[token_manager_type as u8; 32]);
        let token_id = client.mock_all_auths().register_custom_token(
            &deployer,
            &salt,
            &env.register_stellar_asset_contract_v2(deployer.clone())
                .address(),
            &token_manager_type,
        );

        assert_contract_err!(
            client
                .mock_all_auths()
                .try_transfer_mintership(&token_id, &deployer, &new_minter),
            ContractError::InvalidTokenManagerType
        );
    }
}

#[test]
fn transfer_mintership_fails_with_invalid_token_id() {
    let (env, client, _gateway, _gas_service, _signers) = setup_env();
    let minter = Address::generate(&env);
    let new_minter = Address::generate(&env);

    let invalid_token_id = BytesN::<32>::from_array(&env, &[0u8; 32]);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_mintership(&invalid_token_id, &minter, &new_minter),
        ContractError::InvalidTokenId
    );
}
