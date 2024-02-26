use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contractimpl, log, panic_with_error, symbol_short, Address, Bytes, BytesN, Env, String, Symbol, Vec, U256
};

use crate::error::{self, Error};
use crate::event;
use crate::types::{Proof, WeightedSigners};
use crate::storage_types::DataKey;
use crate::interface::AxelarAuthVerifierInterface;

#[contract]
pub struct AxelarAuthVerifier;

#[contractimpl]
impl AxelarAuthVerifier {
    pub fn initialize(env: Env, previous_signer_retention: u32, operator_set: Bytes) {
        if env
            .storage()
            .instance()
            .has(&DataKey::Initialized)
        {
            panic!("Already initialized");
        }

        env.storage()
            .instance()
            .set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::PreviousSignerRetention, &previous_signer_retention);

        let signer_sets = Vec::<WeightedSigners>::from_xdr(&env, &operator_set).unwrap();

        if signer_sets.is_empty() {
            panic_with_error!(env, Error::InvalidOperators);
        }

        for signer_set in signer_sets {
            Self::rotate_signer_set(&env, signer_set);
        }
    }
}

#[contractimpl]
impl AxelarAuthVerifierInterface for AxelarAuthVerifier {
    fn validate_proof(env: &Env, msg_hash: BytesN<32>, proof: Bytes) -> bool {
        let proof = Proof::from_xdr(env, &proof).unwrap();

        let signer_set_hash = env.crypto().keccak256(&proof.signer_set.clone().to_xdr(env));

        let signer_set_epoch: u64 = env.storage().persistent().get(&DataKey::EpochBySignerHash(signer_set_hash.clone())).unwrap();

        let epoch: u64 = env.storage().instance().get(&DataKey::Epoch).unwrap();

        if signer_set_epoch == 0 {
            return false;
        }

        let previous_signer_retention: u32 = env.storage().instance().get(&DataKey::PreviousSignerRetention).unwrap();

        if epoch - signer_set_epoch > previous_signer_retention as u64 {
            return false;
        }

        if !Self::validate_signatures(env, msg_hash, proof) {
            return false;
        }

        epoch == signer_set_epoch
    }

    fn transfer_operatorship(env: Env, new_operator_set: Bytes) {
        // TODO: do we need to check if contract has been initialized?
        let signers = WeightedSigners::from_xdr(&env, &new_operator_set).unwrap();

        Self::rotate_signer_set(&env, signers);
    }
}

impl AxelarAuthVerifier {
    fn rotate_signer_set(env: &Env, new_signer_set: WeightedSigners) {
        if !new_signer_set.is_valid() {
            panic_with_error!(env, Error::InvalidOperators);
        }

        let new_signer_hash = env.crypto().keccak256(&new_signer_set.clone().to_xdr(env));
        let new_epoch = env.storage().instance().get::<DataKey, u64>(&DataKey::Epoch).unwrap() + 1;

        env.storage().instance().set(&DataKey::Epoch, &new_epoch);

        env.storage()
            .persistent()
            .set(&DataKey::SignerHashByEpoch(new_epoch), &new_signer_hash);

        env.storage()
            .persistent()
            .set(&DataKey::EpochBySignerHash(new_signer_hash.clone()), &new_epoch);

        event::transfer_operatorship(env, new_signer_set, new_signer_hash);
    }

    fn validate_signatures(env: &Env, msg_hash: BytesN<32>, proof: Proof) -> bool {
        let Proof { signer_set, signatures } = proof;

        if signatures.is_empty() {
            return false;
        }

        let total_weight = U256::from_u32(env, 0);
        let mut signer_index = 0;

        for (signature, recovery_id) in signatures.into_iter() {
            // TODO: typo in recovery id name
            let pub_key = env.crypto().secp256k1_recover(&msg_hash, &signature, recovery_id);
            let expected_signer = env.crypto().keccak256(&pub_key.into());

            loop {
                let (signer, weight) = signer_set.signers.get(signer_index).unwrap();

                if expected_signer == signer {
                    total_weight.add(&weight);

                    if total_weight >= signer_set.threshold {
                        return true;
                    }

                    break;
                }

                signer_index += 1;

                if signer_index == signer_set.signers.len() {
                    return false;
                }
            }
        }

        false
    }
}

impl WeightedSigners {
    pub fn is_valid(&self) -> bool {
        if self.signers.is_empty() {
            return false;
        }

        // TODO: zero address check?

        let first_weight = self.signers.get(0).unwrap();
        let env = first_weight.1.env();
        let zero = U256::from_u32(env, 0);
        let total_weight = zero.clone();

        for weight in self.signers.iter().map(|s| s.1) {
            if weight == zero {
                return false;
            }

            total_weight.add(&weight);
        }

        if self.threshold == zero || total_weight < self.threshold {
            return false;
        }

        let mut previous_signer = self.signers.get(0).unwrap().0;

        for signer in self.signers.iter().skip(1).map(|s| s.0) {
            if signer <= previous_signer {
                return false;
            }

            previous_signer = signer;
        }

        true
    }
}
