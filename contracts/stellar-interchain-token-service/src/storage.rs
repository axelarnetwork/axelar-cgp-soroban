use stellar_axelar_std::{contractstorage, contracttype, soroban_sdk, Address, BytesN, String};

use crate::types::TokenManagerType;

#[contractstorage]
enum DataKey {
    #[instance]
    #[value(Address)]
    Gateway,

    #[instance]
    #[value(Address)]
    GasService,

    #[instance]
    #[value(String)]
    ChainName,

    #[instance]
    #[value(String)]
    ItsHubAddress,

    #[instance]
    #[value(Address)]
    NativeTokenAddress,

    #[instance]
    #[value(BytesN<32>)]
    InterchainTokenWasmHash,

    #[instance]
    #[value(BytesN<32>)]
    TokenManagerWasmHash,

    #[persistent]
    #[status]
    TrustedChain { chain: String },

    #[persistent]
    #[value(TokenIdConfigValue)]
    TokenIdConfig { token_id: BytesN<32> },

    #[persistent]
    #[value(i128)]
    FlowLimit { token_id: BytesN<32> },

    #[persistent]
    #[status]
    FlowLimiter {
        token_id: BytesN<32>,
        flow_limiter: Address,
    },

    #[temporary]
    #[value(i128)]
    FlowOut { token_id: BytesN<32>, epoch: u64 },

    #[temporary]
    #[value(i128)]
    FlowIn { token_id: BytesN<32>, epoch: u64 },
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenIdConfigValue {
    pub token_address: Address,
    pub token_manager: Address,
    pub token_manager_type: TokenManagerType,
}
