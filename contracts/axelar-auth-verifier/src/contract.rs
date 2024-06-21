use core::panic;

use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, crypto::Hash, Env, Vec, U256};

use crate::error::Error;
use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_interfaces::{
    axelar_auth_verifier::AxelarAuthVerifierInterface,
    types::{Proof, WeightedSigner, WeightedSigners},
};

#[contract]
pub struct AxelarAuthVerifier;

#[contractimpl]
impl AxelarAuthVerifier {
    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }

    pub fn owner(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Owner).unwrap()
    }
}

#[contractimpl]
impl AxelarAuthVerifierInterface for AxelarAuthVerifier {
    fn initialize(
        env: Env,
        owner: Address,
        previous_signer_retention: u64,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        initial_signers: Vec<WeightedSigners>,
    ) {
        if env.storage().instance().has(&DataKey::Initialized) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Epoch, &0_u64);

        env.storage().instance().set(&DataKey::Owner, &owner);

        // TODO: Do we need to manually expose these in a query, or can it be read directly off of storage in Stellar?
        env.storage().instance().set(
            &DataKey::PreviousSignerRetention,
            &previous_signer_retention,
        );

        env.storage()
            .instance()
            .set(&DataKey::DomainSeparator, &domain_separator);

        env.storage()
            .instance()
            .set(&DataKey::MinimumRotationDelay, &minimum_rotation_delay);

        if initial_signers.is_empty() {
            panic_with_error!(env, Error::InvalidOperators);
        }

        for signers in initial_signers.into_iter() {
            Self::rotate_signer_set(&env, signers, false);
        }
    }

    fn epoch(env: Env) -> u64 {
        env.storage().instance().get(&DataKey::Epoch).unwrap()
    }

    fn validate_proof(env: Env, data_hash: BytesN<32>, proof: Proof) -> bool {
        let signer_hash: BytesN<32> = env.crypto().keccak256(&proof.signers.clone().to_xdr(&env)).into();

        let signer_epoch: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EpochBySignerHash(signer_hash.clone()))
            .unwrap_or(0);

        if signer_epoch == 0 {
            panic!("invalid epoch");
        }

        let epoch: u64 = env.storage().instance().get(&DataKey::Epoch).unwrap();

        let previous_signer_retention: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PreviousSignerRetention)
            .unwrap();

        if epoch - signer_epoch > previous_signer_retention {
            panic_with_error!(env, Error::OutdatedSigners);
        }

        let msg_hash = Self::message_hash_to_sign(&env, signer_hash, data_hash);

        if !Self::validate_signatures(&env, msg_hash, proof) {
            panic!("invalid signatures");
        }

        epoch == signer_epoch
    }

    fn rotate_signers(env: Env, new_signers: WeightedSigners, enforce_rotation_delay: bool) {
        // TODO: do we need to check explicitly if contract has been initialized?

        // Only allow owner to transfer operatorship. This is meant to be set to the gateway contract
        let owner: Address = env.storage().instance().get(&DataKey::Owner).unwrap();
        owner.require_auth();

        Self::rotate_signer_set(&env, new_signers, enforce_rotation_delay);
    }
}

impl AxelarAuthVerifier {
    fn message_hash_to_sign(env: &Env, signer_hash: BytesN<32>, data_hash: BytesN<32>) -> Hash<32> {
        let domain_separator: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::DomainSeparator)
            .unwrap();

        let mut msg: Bytes = domain_separator.into();
        msg.extend_from_array(&signer_hash.to_array());
        msg.extend_from_array(&data_hash.to_array());

        // TODO: use an appropriate non tx overlapping prefix
        env.crypto().keccak256(&msg)
    }

    fn rotate_signer_set(env: &Env, new_signers: WeightedSigners, enforce_rotation_delay: bool) {
        if !validate_signers(env, &new_signers) {
            panic_with_error!(env, Error::InvalidOperators);
        }

        Self::update_rotation_timestamp(env, enforce_rotation_delay);

        let new_signer_hash: BytesN<32> = env.crypto().keccak256(&new_signers.clone().to_xdr(env)).into();
        let new_epoch: u64 = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::Epoch)
            .unwrap()
            + 1;

        env.storage().instance().set(&DataKey::Epoch, &new_epoch);

        env.storage()
            .persistent()
            .set(&DataKey::SignerHashByEpoch(new_epoch), &new_signer_hash);

        // If new_signers has been rotated to before, we will overwrite the epoch to point to the latest
        env.storage().persistent().set(
            &DataKey::EpochBySignerHash(new_signer_hash.clone()),
            &new_epoch,
        );

        event::rotate_signers(env, new_epoch, new_signers, new_signer_hash);
    }

    fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) {
        let minimum_rotation_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MinimumRotationDelay)
            .unwrap();

        let last_rotation_timestamp: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LastRotationTimestamp)
            .unwrap_or(0);

        let current_timestamp = env.ledger().timestamp();

        if enforce_rotation_delay
            && (current_timestamp - last_rotation_timestamp < minimum_rotation_delay)
        {
            panic_with_error!(env, Error::InsufficientRotationDelay);
        }

        env.storage()
            .instance()
            .set(&DataKey::LastRotationTimestamp, &current_timestamp);
    }

    fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
        let Proof {
            signers,
            signatures,
        } = proof;

        if signatures.is_empty() {
            return false;
        }

        let mut total_weight = U256::from_u32(env, 0);
        let mut signer_index = 0;

        for (signature, recovery_id) in signatures.into_iter() {
            // TODO: check if any additional validation is needed for signature and output of ec recover, or if it's fully handled by the sdk
            let pub_key = env
                .crypto()
                .secp256k1_recover(&msg_hash, &signature, recovery_id);
            let expected_signer: BytesN<32> = env.crypto().keccak256(&pub_key.into()).into();

            while signer_index < signers.signers.len() {
                let WeightedSigner { signer, .. } = signers.signers.get(signer_index).unwrap();

                if expected_signer == signer {
                    break;
                }

                signer_index += 1;
            }

            if signer_index == signers.signers.len() {
                return false;
            }

            let WeightedSigner { weight, .. } = signers.signers.get(signer_index).unwrap();

            total_weight = total_weight.add(&weight);

            if total_weight >= signers.threshold {
                return true;
            }
        }

        false
    }
}

/// Check if signer set is valid, i.e signer/pub key hash are in sorted order,
/// weights are non-zero and sum to at least threshold
pub fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) -> bool {
    if weighted_signers.signers.is_empty() {
        return false;
    }

    // TODO: what's the min address/hash?
    let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
    let zero = U256::from_u32(env, 0);
    let mut total_weight = zero.clone();

    for signer in weighted_signers.signers.iter() {
        if previous_signer >= signer.signer {
            return false;
        }

        if signer.weight == zero {
            return false;
        }

        previous_signer = signer.signer;
        total_weight = total_weight.add(&signer.weight);
    }

    if weighted_signers.threshold == zero || total_weight < weighted_signers.threshold {
        return false;
    }

    true
}
