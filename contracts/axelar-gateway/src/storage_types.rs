use soroban_sdk::{contracttype, String};

use axelar_soroban_std::types::Hash;

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
    Approved(Hash),
    Executed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    AuthModule,
    Operator,
    MessageApproval(MessageApprovalKey),
    RotationExecuted(Hash),
}
