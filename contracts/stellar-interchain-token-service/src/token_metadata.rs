extern crate alloc;

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

    fn new_normalized(
        env: &Env,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<Self, ContractError>;

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

    /// Builds token metadata from an inbound cross-chain deployment, normalizing it instead of
    /// rejecting it where possible.
    ///
    /// Remote chains follow different naming rules, and the metadata of an inbound deployment is
    /// fixed in an already-approved cross-chain message: it cannot be corrected and retried, and
    /// once the ITS Hub has routed a remote deployment it forbids further ones for that token. So
    /// rejecting metadata here makes the token permanently un-onboardable to Stellar.
    ///
    /// Accordingly, an over-long name or symbol is truncated rather than rejected, and non-ASCII
    /// is accepted: the ABI decodes both fields from a Solidity `string` into a Rust `String`, so
    /// they are already valid UTF-8 by construction. Contrast [`Self::new`], used for local
    /// deployments, where the caller supplies a Soroban `String` that carries no encoding
    /// guarantee at all and can be corrected and resubmitted.
    ///
    /// An empty name or symbol is still rejected — there is nothing to normalize it to, and
    /// substituting a placeholder would mean inventing metadata that matches nothing on the
    /// source chain.
    fn new_normalized(
        env: &Env,
        name: String,
        symbol: String,
        decimals: u32,
    ) -> Result<Self, ContractError> {
        let token_metadata = Self {
            name: truncate_utf8(env, name, MAX_NAME_LENGTH),
            symbol: truncate_utf8(env, symbol, MAX_SYMBOL_LENGTH),
            decimal: decimals,
        };

        ensure!(
            token_metadata.decimal <= MAX_DECIMALS,
            ContractError::InvalidTokenDecimals
        );
        ensure!(
            !token_metadata.name.is_empty(),
            ContractError::InvalidTokenName
        );
        ensure!(
            !token_metadata.symbol.is_empty(),
            ContractError::InvalidTokenSymbol
        );

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

/// Truncates `s` to at most `max_bytes`, cutting at a UTF-8 character boundary so a multi-byte
/// character is never split. Mirrors Solana ITS's `truncate_utf8`.
fn truncate_utf8(env: &Env, s: String, max_bytes: u32) -> String {
    if s.len() <= max_bytes {
        return s;
    }

    let mut bytes = alloc::vec![0u8; s.len() as usize];
    s.copy_into_slice(&mut bytes);

    // A UTF-8 continuation byte matches 0b10xxxxxx. Step back off any continuation byte so the
    // cut lands on the start of a character.
    let mut cut = max_bytes as usize;
    while cut > 0 && (bytes[cut] & 0xC0) == 0x80 {
        cut -= 1;
    }

    String::from_bytes(env, &bytes[..cut])
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

#[cfg(test)]
mod normalized_tests {
    use stellar_axelar_std::assert_ok;

    use super::*;

    fn long_ascii(len: usize) -> alloc::string::String {
        core::iter::repeat_n('a', len).collect()
    }

    #[test]
    fn new_normalized_keeps_valid_metadata_unchanged() {
        let env = Env::default();

        let metadata = assert_ok!(TokenMetadata::new_normalized(
            &env,
            String::from_str(&env, "Test Token"),
            String::from_str(&env, "TST"),
            18,
        ));

        assert_eq!(metadata.name, String::from_str(&env, "Test Token"));
        assert_eq!(metadata.symbol, String::from_str(&env, "TST"));
        assert_eq!(metadata.decimal, 18);
    }

    #[test]
    fn new_normalized_accepts_non_ascii() {
        let env = Env::default();

        let name = String::from_str(&env, "世界コイン");
        let symbol = String::from_str(&env, "世界");

        let metadata = assert_ok!(TokenMetadata::new_normalized(
            &env,
            name.clone(),
            symbol.clone(),
            7,
        ));

        assert_eq!(metadata.name, name);
        assert_eq!(metadata.symbol, symbol);
    }

    #[test]
    fn new_normalized_truncates_over_long_name_and_symbol() {
        let env = Env::default();

        let metadata = assert_ok!(TokenMetadata::new_normalized(
            &env,
            String::from_str(&env, &long_ascii(MAX_NAME_LENGTH as usize + 8)),
            String::from_str(&env, &long_ascii(MAX_SYMBOL_LENGTH as usize + 8)),
            7,
        ));

        assert_eq!(metadata.name.len(), MAX_NAME_LENGTH);
        assert_eq!(metadata.symbol.len(), MAX_SYMBOL_LENGTH);
    }

    #[test]
    fn new_normalized_truncates_multi_byte_name_on_char_boundary() {
        let env = Env::default();

        // '界' is 3 bytes, so 11 of them is 33 bytes: one byte over the limit. Cutting at 32
        // would split the last character, so the whole character must be dropped.
        let name: alloc::string::String = core::iter::repeat_n('界', 11).collect();
        assert_eq!(name.len(), 33);

        let metadata = assert_ok!(TokenMetadata::new_normalized(
            &env,
            String::from_str(&env, &name),
            String::from_str(&env, "TST"),
            7,
        ));

        // 10 characters, 30 bytes — the largest character boundary at or below 32.
        assert_eq!(metadata.name.len(), 30);
        assert_eq!(
            metadata.name,
            String::from_str(&env, &long_multi_byte_prefix(10))
        );
    }

    fn long_multi_byte_prefix(chars: usize) -> alloc::string::String {
        core::iter::repeat_n('界', chars).collect()
    }

    #[test]
    fn new_normalized_fails_with_empty_name() {
        let env = Env::default();

        let result = TokenMetadata::new_normalized(
            &env,
            String::from_str(&env, ""),
            String::from_str(&env, "TST"),
            7,
        );

        assert!(matches!(result, Err(ContractError::InvalidTokenName)));
    }

    #[test]
    fn new_normalized_fails_with_empty_symbol() {
        let env = Env::default();

        let result = TokenMetadata::new_normalized(
            &env,
            String::from_str(&env, "Test Token"),
            String::from_str(&env, ""),
            7,
        );

        assert!(matches!(result, Err(ContractError::InvalidTokenSymbol)));
    }
}
