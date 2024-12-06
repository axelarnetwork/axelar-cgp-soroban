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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    InterchainTransfer(InterchainTransfer),
    DeployInterchainToken(DeployInterchainToken),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransfer {
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeployInterchainToken {
    pub token_id: BytesN<32>,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub minter: Option<Bytes>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HubMessage {
    SendToHub {
        destination_chain: String,
        message: Message,
    },
    ReceiveFromHub {
        source_chain: String,
        message: Message,
    },
}
