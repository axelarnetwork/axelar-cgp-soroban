use interchain_token::InterchainTokenClient;
use soroban_sdk::{Env, Address};
use soroban_sdk::token::{StellarAssetClient, TokenClient};

use crate::error::ContractError;
use crate::storage_types::TokenIdConfigValue;
use crate::types::TokenManagerType;

pub fn take_token(env: &Env, sender: Address, TokenIdConfigValue { token_address, token_manager_type }: TokenIdConfigValue, amount: i128) -> Result<(), ContractError> {
    sender.require_auth();

    let token = TokenClient::new(env, &token_address);

    match token_manager_type {
        TokenManagerType::NativeInterchainToken => token.burn(&sender, &amount),
        TokenManagerType::LockUnlock => token.transfer(&sender, &env.current_contract_address(), &amount),
    }

    Ok(())
}

pub fn give_token(env: &Env, recipient: Address, TokenIdConfigValue { token_address, token_manager_type }: TokenIdConfigValue, amount: i128) -> Result<(), ContractError> {
    match token_manager_type {
        TokenManagerType::NativeInterchainToken => InterchainTokenClient::new(env, &token_address).mint(&env.current_contract_address(), &recipient, &amount),
        TokenManagerType::LockUnlock => TokenClient::new(env, &token_address).transfer(&env.current_contract_address(), &recipient, &amount),
    }

    Ok(())
}
