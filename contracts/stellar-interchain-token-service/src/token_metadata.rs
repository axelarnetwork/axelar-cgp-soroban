use soroban_token_sdk::metadata::TokenMetadata;
use stellar_axelar_std::string::StringExt;
use stellar_axelar_std::{ensure, token, Address, Env, String};

use crate::error::ContractError;

const NATIVE_TOKEN_NAME: &str = "Stellar";
const NATIVE_TOKEN_SYMBOL: &str = "XLM";
const MAX_DECIMALS: u32 = u8::MAX as u32;
const MAX_NAME_LENGTH: u32 = 32;
const MAX_SYMBOL_LENGTH: u32 = 32;

pub trait TokenMetadataExt: Sized {
    fn new(name: String, symbol: String, decimals: u32) -> Result<Self, ContractError>;

    fn validate(&self) -> Result<(), ContractError>;
}

impl TokenMetadataExt for TokenMetadata {
    fn new(name: String, symbol: String, decimals: u32) -> Result<Self, ContractError> {
        let token_metadata = Self {
            name,
            symbol,
            decimal: decimals,
        };

        token_metadata.validate()?;

        Ok(token_metadata)
    }

    fn validate(&self) -> Result<(), ContractError> {
        ensure!(
            self.decimal <= MAX_DECIMALS,
            ContractError::InvalidTokenDecimals
        );
        ensure!(
            !self.name.is_empty() && self.name.len() <= MAX_NAME_LENGTH,
            ContractError::InvalidTokenName
        );
        ensure!(
            !self.symbol.is_empty() && self.symbol.len() <= MAX_SYMBOL_LENGTH,
            ContractError::InvalidTokenSymbol
        );
        ensure!(&self.name.is_ascii(), ContractError::InvalidTokenName);
        ensure!(&self.symbol.is_ascii(), ContractError::InvalidTokenSymbol);

        Ok(())
    }
}

pub fn token_metadata(
    env: &Env,
    token_address: &Address,
    native_token_address: &Address,
) -> Result<TokenMetadata, ContractError> {
    let token = token::Client::new(env, token_address);
    let decimals = token
        .try_decimals()
        .map_err(|_| ContractError::InvalidTokenAddress)?
        .map_err(|_| ContractError::TokenInvocationError)?;

    if token_address == native_token_address {
        // Stellar's native token SAC reports both its name and symbol as the literal 'native',
        // which is ambiguous in a cross-chain context where it could refer to any chain's native
        // asset. Override them so the token has an unambiguous identity when represented on
        // remote chains. This is a deliberate divergence from the SAC's on-chain metadata; only
        // `decimals` is read from the SAC. Returning early also avoids the `name()`/`symbol()`
        // cross-contract calls below, whose results would be discarded anyway.
        let name = String::from_str(env, NATIVE_TOKEN_NAME);
        let symbol = String::from_str(env, NATIVE_TOKEN_SYMBOL);

        return TokenMetadata::new(name, symbol, decimals);
    }

    let name = token
        .try_name()
        .map_err(|_| ContractError::InvalidTokenAddress)?
        .map_err(|_| ContractError::TokenInvocationError)?;
    let symbol = token
        .try_symbol()
        .map_err(|_| ContractError::InvalidTokenAddress)?
        .map_err(|_| ContractError::TokenInvocationError)?;

    let (name, symbol) = if name.len() > MAX_NAME_LENGTH {
        (symbol.clone(), symbol)
    } else {
        (name, symbol)
    };

    TokenMetadata::new(name, symbol, decimals)
}

#[cfg(test)]
mod tests {
    use stellar_axelar_std::assert_ok;

    use super::*;

    #[test]
    fn token_metadata_new_succeeds() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        assert_ok!(TokenMetadata::new(name, symbol, decimals));
    }

    #[test]
    fn token_metadata_new_fails_with_invalid_ascii_name() {
        let env = Env::default();

        let name = String::from_str(&env, "Test世界！");
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        // TODO: use assert_err! once TokenMetadata implements Debug trait in new release
        assert!(matches!(result, Err(ContractError::InvalidTokenName)));
    }

    #[test]
    fn token_metadata_new_fails_with_invalid_ascii_symbol() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "Test世界！");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        // TODO: use assert_err! once TokenMetadata implements Debug trait in new release
        assert!(matches!(result, Err(ContractError::InvalidTokenSymbol)));
    }

    #[test]
    fn token_metadata_new_fails_with_empty_name() {
        let env = Env::default();

        let name = String::from_str(&env, "");
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenName)));
    }

    #[test]
    fn token_metadata_new_fails_with_empty_symbol() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenSymbol)));
    }

    #[test]
    fn token_metadata_new_fails_with_too_long_name() {
        let env = Env::default();

        let long_name = "A".repeat(MAX_NAME_LENGTH as usize + 1);
        let name = String::from_str(&env, &long_name);
        let symbol = String::from_str(&env, "Test");
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenName)));
    }

    #[test]
    fn token_metadata_new_fails_with_too_long_symbol() {
        let env = Env::default();

        let long_symbol = "A".repeat(MAX_SYMBOL_LENGTH as usize + 1);
        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, &long_symbol);
        let decimals = 18;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenSymbol)));
    }

    #[test]
    fn token_metadata_new_fails_with_invalid_decimals() {
        let env = Env::default();

        let name = String::from_str(&env, "Test");
        let symbol = String::from_str(&env, "Test");
        let decimals = MAX_DECIMALS + 1;

        let result = TokenMetadata::new(name, symbol, decimals);
        assert!(matches!(result, Err(ContractError::InvalidTokenDecimals)));
    }

    #[test]
    fn token_metadata_new_succeeds_with_max_values() {
        let env = Env::default();

        let name = String::from_str(&env, &"A".repeat(MAX_NAME_LENGTH as usize));
        let symbol = String::from_str(&env, &"A".repeat(MAX_SYMBOL_LENGTH as usize));
        let decimals = MAX_DECIMALS;

        assert_ok!(TokenMetadata::new(name, symbol, decimals));
    }
}
