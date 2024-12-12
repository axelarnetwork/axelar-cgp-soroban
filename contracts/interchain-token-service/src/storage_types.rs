use soroban_sdk::{contracttype, Address, BytesN, String};

use crate::types::TokenManagerType;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    TrustedChain(String),
    Gateway,
    GasService,
    ItsHubAddress,
    ChainName,
    InterchainTokenWasmHash,
    TokenId(BytesN<32>),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIdConfig {
    pub token_address: Address,
    pub token_manager_type: TokenManagerType,
}
