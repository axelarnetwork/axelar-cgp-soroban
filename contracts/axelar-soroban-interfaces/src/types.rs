use soroban_sdk::{contracttype, xdr::ToXdr, Address, BytesN, Env, String, Vec};

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

/// `ProofSignature` represents an optional signature from a signer.
/// Since Soroban doesn't support use of `Option` in it's contract interfaces,
/// we use this enum instead.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProofSignature {
    Signed(BytesN<64>), // Ed25519 signature
    Unsigned,
}

/// `ProofSigner` represents a signer in a proof. If the signer submitted a signature,
/// and if it is being included in the proof to meet the threshold, then a `ProofSignature` is attached.
/// Otherwise, the `ProofSignature` is `Unsigned`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProofSigner {
    pub signer: WeightedSigner,
    pub signature: ProofSignature,
}

/// `Proof` represents a proof that a set of signers have signed a message.
/// All weighted signers are included in the along with a signature, if they have signed the message,
/// until threshold is met.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Proof {
    pub signers: Vec<ProofSigner>,
    pub threshold: u128,
    pub nonce: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CommandType {
    ApproveMessages,
    RotateSigners,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Message {
    pub source_chain: String,
    pub message_id: String,
    pub source_address: String,
    pub contract_address: Address,
    pub payload_hash: BytesN<32>,
}

impl WeightedSigners {
    pub fn hash(&self, env: &Env) -> BytesN<32> {
        env.crypto().keccak256(&self.clone().to_xdr(env)).into()
    }

    pub fn signers_rotation_hash(&self, env: &Env) -> BytesN<32> {
        env.crypto()
            .keccak256(&(CommandType::RotateSigners, self.clone()).to_xdr(env))
            .into()
    }
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
