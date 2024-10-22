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

#[cfg(test)]
mod tests {
    use hex_literal::hex;
    use soroban_sdk::{vec, BytesN, Env, Vec};

    use crate::types::{WeightedSigner, WeightedSigners};
    extern crate std;

    #[test]
    fn weighted_signers_hash() {
        let env = Env::default();

        let signers_array = [
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("0a245a2a2a5e8ec439d1377579a08fc78ea55647ba6fcb1f5d8a360218e8a985"),
                ),
                weight: 3,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("0b422cf449d900f6f8eb97f62e35811c62eb75feb84dfccef44a5c1c3dbac2ad"),
                ),
                weight: 2,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("18c34bf01a11b5ba21ea11b1678f3035ef753f0bdb1d5014ec21037e8f99e2a2"),
                ),
                weight: 4,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("f683ca8a6d7fe55f25599bb64b01edcc5eeb85fe5b63d3a4f0b3c32405005518"),
                ),
                weight: 4,
            },
            WeightedSigner {
                signer: BytesN::<32>::from_array(
                    &env,
                    &hex!("fbb4b870e800038f1379697fae3058938c59b696f38dd0fdf2659c0cf3a5b663"),
                ),
                weight: 2,
            },
        ];

        let weighted_signers = WeightedSigners {
            signers: Vec::from_array(&env, signers_array),
            threshold: 8u128,
            nonce: BytesN::<32>::from_array(
                &env,
                &hex!("8784bf7be5a9baaeea47e12d9e8ad0dec29afcbc3617d97f771e3c24fa945dce"),
            ),
        };

        let hash = weighted_signers.hash(&env);
        let signers_rotation_hash = weighted_signers.signers_rotation_hash(&env);

        let expected_hash = BytesN::<32>::from_array(
            &env,
            &hex!("18b1ff823e202dab87cada621717e5be4955734bb973151eb489e6f1576ce3d4"),
        );

        let expected_signers_rotation_hash = BytesN::<32>::from_array(
            &env,
            &hex!("4ad8f3015146ac68334fd405f90e6ca75fbf2c276b333a8747c9ba83d9c3f1f6"),
        );

        //assert_eq!(hash, expected_hash);
        //assert_eq!(signers_rotation_hash, expected_signers_rotation_hash);

        goldie::assert_json!(&vec![hash, signers_rotation_hash]);
    }
}
