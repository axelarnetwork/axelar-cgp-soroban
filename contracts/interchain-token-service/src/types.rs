use soroban_sdk::{contracttype, Bytes, BytesN, String};

#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum MessageType {
    InterchainTransfer = 0,
    DeployInterchainToken = 1,
    DeployTokenManager = 2, // note, this case is not supported by the ITS hub
    SendToHub = 3,
    ReceiveFromHub = 4,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    InterchainTransfer(InterchainTransfer),
    DeployInterchainToken(DeployInterchainToken),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HubMessage {
    SendToHub(SendToHub),
    ReceiveFromHub(ReceiveFromHub),
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransfer {
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
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
pub struct SendToHub {
    pub destination_chain: String,
    pub message: Message,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReceiveFromHub {
    pub source_chain: String,
    pub message: Message,
}
