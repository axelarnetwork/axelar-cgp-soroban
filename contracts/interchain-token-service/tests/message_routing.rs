mod utils;
use axelar_soroban_std::assert_contract_err;
use interchain_token_service::error::ContractError;
use soroban_sdk::{testutils::Address as _, xdr::ToXdr, Address, BytesN, String};
use utils::{register_chains, setup_env, setup_gas_token};

#[test]
fn send_directly_to_hub_chain_fails() {
    let (env, client, _gateway_client, _) = setup_env();
    register_chains(&env, &client);
    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);

    let result = client.mock_all_auths().try_interchain_transfer(
        &sender,
        &BytesN::from_array(&env, &[255u8; 32]),
        &client.its_hub_chain_name(),
        &Address::generate(&env).to_xdr(&env),
        &i128::MAX,
        &None,
        &gas_token,
    );
    assert_contract_err!(result, ContractError::UntrustedChain);
}

#[test]
fn send_to_untrusted_chain_fails() {
    let (env, client, _gateway_client, _) = setup_env();
    register_chains(&env, &client);
    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);

    let result = client.mock_all_auths().try_interchain_transfer(
        &sender,
        &BytesN::from_array(&env, &[255u8; 32]),
        &String::from_str(&env, "untrusted_chain"),
        &Address::generate(&env).to_xdr(&env),
        &i128::MAX,
        &None,
        &gas_token,
    );
    assert_contract_err!(result, ContractError::UntrustedChain);
}
