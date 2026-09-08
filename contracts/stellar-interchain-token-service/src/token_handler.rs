use stellar_axelar_std::token::TokenClient;
use stellar_axelar_std::{Address, Env};
use stellar_token_manager::TokenManagerClient;

use crate::error::ContractError;
use crate::storage::TokenIdConfigValue;
use crate::token_manager::TokenManagerClientExt;
use crate::types::TokenManagerType;

pub fn take_token(
    env: &Env,
    sender: &Address,
    TokenIdConfigValue {
        token_address,
        token_manager,
        token_manager_type,
    }: TokenIdConfigValue,
    amount: i128,
) -> Result<(), ContractError> {
    let token = TokenClient::new(env, &token_address);

    match token_manager_type {
        // Burn tokens directly from the sender
        TokenManagerType::NativeInterchainToken | TokenManagerType::MintBurn => {
            token.burn(sender, &amount)
        }

        // In EVM, `MintBurnFrom` would require explicit approval (allowance) before burning.
        // However, in Stellar's account abstraction model, when a user signs a transaction,
        // they can authorize all sub-invocations within that transaction by default.
        // Therefore, we can directly burn from the sender without requiring a separate approval,
        // as the user has already authorized this action by signing the transaction.
        TokenManagerType::MintBurnFrom => token.burn(sender, &amount),

        // Transfer tokens from the sender to the token manager to lock them
        TokenManagerType::LockUnlock => token.transfer(sender, &token_manager, &amount),
    }

    Ok(())
}

pub fn give_token(
    env: &Env,
    recipient: &Address,
    TokenIdConfigValue {
        token_address,
        token_manager,
        token_manager_type,
    }: TokenIdConfigValue,
    amount: i128,
) -> Result<(), ContractError> {
    let token_manager = TokenManagerClient::new(env, &token_manager);

    match token_manager_type {
        // For NativeInterchainToken and MintBurnFrom,
        // `mint_from` interface allows the token to potentially have multiple minters, one of them being the token manager to mint tokens for ITS
        TokenManagerType::NativeInterchainToken | TokenManagerType::MintBurnFrom => {
            token_manager.mint_from(env, &token_address, recipient, amount)
        }

        // Transfer previously locked tokens from the token manager to the recipient
        TokenManagerType::LockUnlock => {
            token_manager.transfer(env, &token_address, recipient, amount)
        }

        // For MintBurn, use direct mint where the token manager mints new tokens
        // This assumes the token manager has minting or admin privileges on the token contract, since `mint` interface doesn't indicate who the caller is
        TokenManagerType::MintBurn => token_manager.mint(env, &token_address, recipient, amount),
    }

    Ok(())
}
