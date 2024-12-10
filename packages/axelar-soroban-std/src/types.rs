use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub address: Address, // TODO: check if this can be changed to a TokenClient type instead which is richer than Address, or a generic type implementing TokenInterface
    pub amount: i128,
}

const ZERO_ADDRESS: &str = "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAWHF";

/// Returns Stellar's "dead" address, represented by the constant `ZERO_ADDRESS`.
///
/// # Reference
/// - Stellar GitHub: https://github.com/stellar/js-stellar-base/blob/master/test/unit/address_test.js
pub fn zero_address(env: &Env) -> Address {
    Address::from_string(&String::from_str(env, ZERO_ADDRESS))
}
