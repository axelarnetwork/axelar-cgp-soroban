use soroban_sdk::{contracttype, BytesN, String};

#[contracttype]
#[derive(Clone, Debug)]
pub struct MessageApprovalKey {
    pub message_id: String,
    pub source_chain: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageApprovalValue {
    NotApproved,
    Approved(BytesN<32>),
    Executed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    AuthModule,
    Operator,
    MessageApproval(MessageApprovalKey),
    RotationExecuted(BytesN<32>),
}
