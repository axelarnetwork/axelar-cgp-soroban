mod utils;

use axelar_soroban_std::assert_contract_err;
use axelar_soroban_std::auth_invocation;
use axelar_soroban_std::events;
use interchain_token_service::error::ContractError;
use interchain_token_service::event::InterchainTokenDeploymentStartedEvent;
use interchain_token_service::types::DeployInterchainToken;
use interchain_token_service::types::HubMessage;
use interchain_token_service::types::Message;
use soroban_sdk::testutils::AuthorizedFunction;
use soroban_sdk::testutils::AuthorizedInvocation;
use soroban_sdk::Bytes;
use soroban_sdk::IntoVal;
use soroban_sdk::Symbol;
use soroban_sdk::{testutils::Address as _, Address, BytesN, String};
use soroban_token_sdk::metadata::TokenMetadata;
use utils::setup_env;
use utils::setup_gas_token;
use utils::TokenMetadataExt;

#[test]
fn deploy_remote_interchain_token_succeeds() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = TokenMetadata::new(&env, "name", "symbol", 6);
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
fn deploy_remote_interchain_token_auth_test() {
    let (env, client, _, gas_service, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = TokenMetadata::new(&env, "name", "symbol", 6);
    let initial_supply = 1;

    let token_id = client.mock_all_auths().deploy_interchain_token(
        &sender,
        &salt,
        &token_meta_data,
        &initial_supply,
        &minter,
    );

    let destination_chain = String::from_str(&env, "ethereum");
    let its_hub_chain = String::from_str(&env, "ethereum");
    let its_hub_address = String::from_str(&env, "its_hub_address");

    client
        .mock_all_auths()
        .set_trusted_chain(&destination_chain);

    let deployed_token_id = client.mock_all_auths().deploy_remote_interchain_token(
        &sender,
        &salt,
        &destination_chain,
        &gas_token.clone(),
    );
    assert_eq!(token_id, deployed_token_id);

    let message = Message::DeployInterchainToken(DeployInterchainToken {
        token_id: token_id.clone(),
        name: token_meta_data.name.clone(),
        symbol: token_meta_data.symbol.clone(),
        decimals: token_meta_data.decimal as u8,
        minter: None,
    });

    let payload = HubMessage::SendToHub {
        destination_chain: destination_chain.clone(),
        message,
    }
    .abi_encode(&env);

    let transfer_auth = auth_invocation!(&env,
        "transfer",
        gas_token.clone().address =>
        (
            &sender,
            gas_service.address.clone(),
            gas_token.amount
        )
    );

    let pay_gas_auth = auth_invocation!(&env,
        "pay_gas",
        gas_service.address =>
        (
            client.address.clone(),
            its_hub_chain.clone(),
            its_hub_address.clone(),
            payload.clone(),
            &sender,
            gas_token.clone(),
            &Bytes::new(&env)
        ),
        transfer_auth
    );

    let deploy_remote_interchain_token_auth = auth_invocation!(&env,
        sender,
        "deploy_remote_interchain_token",
        client.address.clone() =>
        (
            &sender,
            salt,
            destination_chain,
            gas_token
        ),
        pay_gas_auth
    );

    assert_eq!(env.auths(), deploy_remote_interchain_token_auth);
}

#[test]
fn deploy_remote_interchain_token_fails_untrusted_chain() {
    let (env, client, _, _, _) = setup_env();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let minter: Option<Address> = None;
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);
    let token_meta_data = TokenMetadata::new(&env, "name", "symbol", 6);
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
fn deploy_remote_interchain_token_fails_with_invalid_token_id() {
    let (env, client, _, _, _) = setup_env();
    env.mock_all_auths();

    let sender = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);
    let salt = BytesN::<32>::from_array(&env, &[1; 32]);

    let destination_chain = String::from_str(&env, "ethereum");

    assert_contract_err!(
        client.try_deploy_remote_interchain_token(&sender, &salt, &destination_chain, &gas_token),
        ContractError::InvalidTokenId
    );
}
