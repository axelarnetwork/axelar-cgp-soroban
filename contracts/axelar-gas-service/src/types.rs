use soroban_sdk::{contracttype, Address, BytesN, U256};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refunded {
    pub tx_hash: BytesN<32>,
    pub log_index: U256,
    pub receiver: Address,
    pub token: Address,
    pub amount: U256,
}
