use axelar_soroban_std::assert_contract_err;
use interchain_token_service::error::ContractError;
use interchain_token_service::testutils::{
    bytes_from_hex, register_chains, setup_env, setup_gas_token,
};
use soroban_sdk::{testutils::Address as _, Address, BytesN, String};

#[test]
fn send_directly_to_hub_chain_fails() {
    let (env, client, _gateway_client, _) = setup_env();
    register_chains(&env, &client);
    let sender: Address = Address::generate(&env);
    let gas_token = setup_gas_token(&env, &sender);

    let result = client.try_interchain_transfer(
        &sender,
        &BytesN::from_array(&env, &[255u8; 32]),
        &client.its_hub_chain_name(),
        &bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
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

    let result = client.try_interchain_transfer(
        &sender,
        &BytesN::from_array(&env, &[255u8; 32]),
        &String::from_str(&env, "untrusted_chain"),
        &bytes_from_hex(&env, "4F4495243837681061C4743b74B3eEdf548D56A5"),
        &i128::MAX,
        &None,
        &gas_token,
    );
    assert_contract_err!(result, ContractError::UntrustedChain);
}
