use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractCallApprovalKey {
    pub command_id: BytesN<32>,
    pub source_chain: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct CommandExecutedKey {
    pub command_id: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    CommandExecuted(CommandExecutedKey),
    ContractCallApproval(ContractCallApprovalKey),
}
