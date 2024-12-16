use crate::{ensure, types::TokenError};
use soroban_token_sdk::metadata::TokenMetadata;

pub fn validate_token_metadata(token_metadata: TokenMetadata) -> Result<(), TokenError> {
    ensure!(
        token_metadata.decimal <= u8::MAX.into(),
        TokenError::InvalidDecimal
    );
    ensure!(
        !token_metadata.name.is_empty(),
        TokenError::InvalidTokenName
    );
    ensure!(
        !token_metadata.symbol.is_empty(),
        TokenError::InvalidTokenSymbol
    );
    Ok(())
}
