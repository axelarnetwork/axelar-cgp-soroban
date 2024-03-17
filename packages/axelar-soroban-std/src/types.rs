use soroban_sdk::{contracttype, Address, BytesN};

pub type Hash = BytesN<32>;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenDetails {
    pub token_addr: Address,
    pub amount: i128,
}
