use soroban_sdk::{contracttype, Bytes, BytesN, String};

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    InterchainTransfer = 0,
    DeployInterchainToken = 1,
    DeployTokenManager = 2,
    SendToHub = 3,
    ReceiveFromHub = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenManagerType {
    NativeInterchainToken,
    MintBurnFrom,
    LockUnlock,
    LockUnlockFee,
    MintBurn,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransfer {
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: u128,
    pub data: Bytes,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployInterchainToken {
    pub token_id: BytesN<32>,
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub minter: Option<Bytes>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployTokenManager {
    pub token_id: BytesN<32>,
    pub token_manager_type: TokenManagerType,
    pub params: Bytes,
}
