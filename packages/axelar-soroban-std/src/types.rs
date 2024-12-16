use soroban_sdk::{contracterror, contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub address: Address, // TODO: check if this can be changed to a TokenClient type instead which is richer than Address, or a generic type implementing TokenInterface
    pub amount: i128,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    InvalidDecimal = 0,
    InvalidTokenName = 1,
    InvalidTokenSymbol = 2,
}
