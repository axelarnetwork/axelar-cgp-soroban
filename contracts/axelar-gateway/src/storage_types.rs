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
    /// Gateway
    Initialized,
    AuthModule,
    Operator,
    MessageApproval(MessageApprovalKey),
    RotationExecuted(BytesN<32>),
    /// Auth Module
    AuthInitialized,
    PreviousSignerRetention,
    DomainSeparator,
    MinimumRotationDelay,
    Epoch,
    LastRotationTimestamp,
    SignerHashByEpoch(u64),
    EpochBySignerHash(BytesN<32>),
}
