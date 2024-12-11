use soroban_sdk::{contracttype, BytesN, String};

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
