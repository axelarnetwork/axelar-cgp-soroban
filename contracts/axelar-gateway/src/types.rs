use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallApproval {
    pub source_chain: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    ContractCallApproval(ContractCallApproval),
    TransferOperatorship(Bytes),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandBatch {
    pub chain_id: u64,
    pub commands: Vec<(BytesN<32>, Command)>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedCommandBatch {
    pub batch: CommandBatch,
    pub proof: Bytes,
}
