use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_gas_service::testutils::setup_gas_token;
use stellar_axelar_std::testutils::Address as _;
use stellar_axelar_std::token::{StellarAssetClient, TokenClient};
use stellar_axelar_std::traits::BytesExt;
use stellar_axelar_std::types::Token;
use stellar_axelar_std::{assert_contract_err, events, Address, Bytes, BytesN, Env, String};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::InterchainTransferSentEvent;
use crate::tests::utils::TokenMetadataExt;
use crate::testutils::setup_its_token;
use crate::types::TokenManagerType;
use crate::InterchainTokenServiceClient;

fn dummy_transfer_params(env: &Env) -> (String, Bytes, Option<Bytes>) {
    let destination_chain = String::from_str(env, "ethereum");
    let destination_address = Bytes::from_hex(env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(env, "abcd"));

    (destination_chain, destination_address, data)
}

fn setup_sender(
    env: &Env,
    client: &InterchainTokenServiceClient,
    amount: i128,
) -> (Address, Token, BytesN<32>) {
    let sender: Address = Address::generate(env);
    let gas_token = setup_gas_token(env, &sender);
    let (token_id, _) = setup_its_token(env, client, &sender, amount);

    (sender, gas_token, token_id)
}

#[test]
fn interchain_transfer_send_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let amount = 1000;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, amount);
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));
}

#[test]
fn interchain_transfer_send_succeeds_without_gas_token() {
    let (env, client, _, _, _) = setup_env();

    let amount = 1000;
    let (sender, _, token_id) = setup_sender(&env, &client, amount);
    let gas_token: Option<Token> = None;
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &gas_token,
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -2));
}

#[test]
fn interchain_transfer_canonical_token_send_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let amount = 1000;
    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    let token_address = env
        .register_stellar_asset_contract_v2(sender.clone())
        .address();

    let token_id = client
        .mock_all_auths()
        .register_canonical_token(&token_address);
    let token_manager = client.deployed_token_manager(&token_id);

    assert_eq!(
        TokenClient::new(&env, &token_address).balance(&token_manager),
        0
    );

    StellarAssetClient::new(&env, &token_address)
        .mock_all_auths()
        .mint(&sender, &amount);

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));

    // Check that the tokens were escrowed in the token manager
    assert_eq!(
        TokenClient::new(&env, &token_address).balance(&token_manager),
        amount
    );
}

#[test]
fn interchain_transfer_mint_burn_from_token_send_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let amount = 1000;
    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    let salt = BytesN::<32>::from_array(&env, &[3; 32]);
    let token_manager_type = TokenManagerType::MintBurnFrom;
    let token_metadata = TokenMetadata::new(&env, "Test Token", "TEST", 6);
    let initial_supply = amount * 2;
    let minter = Some(sender.clone());

    let interchain_token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_metadata,
        &initial_supply,
        &minter,
    );

    let interchain_token_address = client.registered_token_address(&interchain_token_id);

    let token_id = client.mock_all_auths().register_custom_token(
        &sender,
        &salt,
        &interchain_token_address,
        &token_manager_type,
    );

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let initial_sender_balance = TokenClient::new(&env, &interchain_token_address).balance(&sender);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));

    let final_sender_balance = TokenClient::new(&env, &interchain_token_address).balance(&sender);
    assert_eq!(final_sender_balance, initial_sender_balance - amount);
}

#[test]
fn interchain_transfer_self_transfer_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let amount = 1000;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, amount);

    let destination_chain = client.chain_name();
    let destination_address = Bytes::from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5");
    let data = Some(Bytes::from_hex(&env, "abcd"));

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &amount,
        &data,
        &Some(gas_token),
    );

    goldie::assert!(events::fmt_emitted_event_at_idx::<
        InterchainTransferSentEvent,
    >(&env, -4));
}

#[test]
fn interchain_transfer_send_fails_when_paused() {
    let (env, client, _, _, _) = setup_env();
    client.mock_all_auths().pause();

    assert_contract_err!(
        client.try_interchain_transfer(
            &Address::generate(&env),
            &BytesN::from_array(&env, &[0u8; 32]),
            &String::from_str(&env, ""),
            &Bytes::from_hex(&env, ""),
            &1,
            &Some(Bytes::from_hex(&env, "")),
            &Some(setup_gas_token(&env, &Address::generate(&env)))
        ),
        ContractError::ContractPaused
    );
}

#[test]
#[should_panic(expected = "burn, Error(Contract, #6)")] // InterchainToken InsufficientBalance
fn interchain_transfer_send_fails_on_insufficient_balance() {
    let (env, client, _, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let amount = 1000;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, amount);
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    client.mock_all_auths().interchain_transfer(
        &sender,
        &token_id,
        &destination_chain,
        &destination_address,
        &(amount + 1),
        &data,
        &Some(gas_token),
    );
}

#[test]
fn interchain_transfer_fails_on_zero_amount() {
    let (env, client, _, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let supply = 1000;
    let transfer_amount = 0;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, supply);
    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &destination_address,
            &transfer_amount,
            &data,
            &Some(gas_token)
        ),
        ContractError::InvalidAmount
    );
}

#[test]
fn interchain_transfer_fails_on_empty_destination_address() {
    let (env, client, _, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let amount = 1000;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, amount);
    let (destination_chain, _, data) = dummy_transfer_params(&env);
    let empty_address = Bytes::new(&env);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &empty_address,
            &amount,
            &data,
            &Some(gas_token)
        ),
        ContractError::InvalidDestinationAddress
    );
}

#[test]
fn interchain_transfer_fails_on_empty_data() {
    let (env, client, _, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let amount = 1000;
    let (sender, gas_token, token_id) = setup_sender(&env, &client, amount);
    let (destination_chain, destination_address, _) = dummy_transfer_params(&env);
    let empty_data = Some(Bytes::new(&env));

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &empty_data,
            &Some(gas_token)
        ),
        ContractError::InvalidData
    );
}

#[test]
fn interchain_transfer_fails_with_invalid_token() {
    let (env, client, _, _, _) = setup_env();
    client
        .mock_all_auths()
        .set_trusted_chain(&client.its_hub_chain_name());

    let amount = 1000;
    let (sender, gas_token, _) = setup_sender(&env, &client, amount);
    let invalid_token_id = BytesN::from_array(&env, &[1u8; 32]);

    let (destination_chain, destination_address, data) = dummy_transfer_params(&env);

    assert_contract_err!(
        client.mock_all_auths().try_interchain_transfer(
            &sender,
            &invalid_token_id,
            &destination_chain,
            &destination_address,
            &amount,
            &data,
            &Some(gas_token)
        ),
        ContractError::InvalidTokenId
    );
}
