use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

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
pub enum ProofSignature {
    Signed(BytesN<64>), // Ed25519 signature
    Unsigned,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofSigner {
    pub signer: WeightedSigner,
    pub signature: ProofSignature,
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

impl Proof {
    /// Get the weighted signers from the proof.
    pub fn weighted_signers(&self) -> WeightedSigners {
        let mut signers = Vec::new(self.signers.env());

        for ProofSigner { signer, .. } in self.signers.iter() {
            signers.push_back(signer);
        }

        WeightedSigners {
            signers,
            threshold: self.threshold,
            nonce: self.nonce.clone(),
        }
    }
}
