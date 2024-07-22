use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigner {
    pub signer: BytesN<32>, // Ed25519 public key
    pub weight: u128,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WeightedSigners {
    pub signers: Vec<WeightedSigner>,
    pub threshold: u128,
    pub nonce: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofSigner {
    pub signer: BytesN<32>, // Ed25519 public key
    pub weight: u128,
    pub signature: Bytes, // Ed25519 signature (empty if not present)
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub signers: Vec<ProofSigner>,
    pub threshold: u128,
    pub nonce: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub message_id: String,
    pub source_chain: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: BytesN<32>,
}
