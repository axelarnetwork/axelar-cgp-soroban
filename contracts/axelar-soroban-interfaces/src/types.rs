use axelar_soroban_std::types::Hash;
use soroban_sdk::{contracttype, Address, String, BytesN, Vec, U256};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigner {
    pub signer: Hash,
    pub weight: U256,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigners {
    pub signers: Vec<WeightedSigner>,
    pub threshold: U256,
    pub nonce: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub signers: WeightedSigners,
    pub signatures: Vec<(BytesN<64>, u32)>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub message_id: String,
    pub source_chain: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: Hash,
}
