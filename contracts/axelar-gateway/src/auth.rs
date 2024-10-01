use axelar_soroban_interfaces::types::{ProofSignature, ProofSigner, WeightedSigner};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{crypto::Hash, panic_with_error, Bytes, BytesN, Env, Vec};

use crate::storage_types::AuthDataKey;
use crate::{error::AuthError, event, auth};
use axelar_soroban_interfaces::types::{Proof, WeightedSigners};

pub(crate) fn init_auth_verifier(
    env: Env,
    domain_separator: BytesN<32>,
    minimum_rotation_delay: u64,
    previous_signer_retention: u64,
    initial_signers: Vec<WeightedSigners>,
) {
    if env.storage().instance().has(&AuthDataKey::Initialized) {
        panic!("Already initialized");
    }

    env.storage().instance().set(&AuthDataKey::Initialized, &true);

    env.storage().instance().set(&AuthDataKey::Epoch, &0_u64);

    // TODO: Do we need to manually expose these in a query, or can it be read directly off of storage in Stellar?
    env.storage().instance().set(
        &AuthDataKey::PreviousSignerRetention,
        &previous_signer_retention,
    );

    env.storage()
        .instance()
        .set(&AuthDataKey::DomainSeparator, &domain_separator);

    env.storage()
        .instance()
        .set(&AuthDataKey::MinimumRotationDelay, &minimum_rotation_delay);

    if initial_signers.is_empty() {
        panic_with_error!(env, AuthError::InvalidSigners);
    }

    for signers in initial_signers.into_iter() {
        auth::rotate_signers_set(&env, signers, false);
    }
}

pub(crate) fn validate_proof(env: &Env, data_hash: BytesN<32>, proof: Proof) -> bool {
    let signer_set = proof.weighted_signers();

    let signer_hash: BytesN<32> = env.crypto().keccak256(&signer_set.to_xdr(&env)).into();

    let signer_epoch: u64 = env
        .storage()
        .persistent()
        .get(&AuthDataKey::EpochBySignerHash(signer_hash.clone()))
        .unwrap_or(0);

    if signer_epoch == 0 {
        panic_with_error!(env, AuthError::InvalidSigners);
    }

    let current_epoch: u64 = auth::epoch(&env);

    let is_latest_signers: bool = signer_epoch == current_epoch;

    let previous_signer_retention: u64 = env
        .storage()
        .instance()
        .get(&AuthDataKey::PreviousSignerRetention)
        .unwrap();

    if current_epoch - signer_epoch > previous_signer_retention {
        panic_with_error!(env, AuthError::InvalidSigners);
    }

    let msg_hash = auth::message_hash_to_sign(&env, signer_hash, data_hash);

    if !auth::validate_signatures(&env, msg_hash, proof.clone()) {
        panic_with_error!(env, AuthError::InvalidSignatures);
    }

    is_latest_signers
}

pub(crate) fn rotate_signers_set(env: &Env, new_signers: WeightedSigners, enforce_rotation_delay: bool) {
    auth::validate_signers(env, &new_signers);

    auth::update_rotation_timestamp(env, enforce_rotation_delay);

    let new_signer_hash: BytesN<32> = env
        .crypto()
        .keccak256(&new_signers.clone().to_xdr(env))
        .into();
    let new_epoch: u64 = auth::epoch(&env) + 1;

    env.storage().instance().set(&AuthDataKey::Epoch, &new_epoch);

    env.storage()
        .persistent()
        .set(&AuthDataKey::SignerHashByEpoch(new_epoch), &new_signer_hash);

    // If new_signers has been rotated to before, we will overwrite the epoch to point to the latest
    env.storage().persistent().set(
        &AuthDataKey::EpochBySignerHash(new_signer_hash.clone()),
        &new_epoch,
    );

    event::rotate_signers_set(env, new_epoch, new_signers, new_signer_hash);
}

fn message_hash_to_sign(env: &Env, signer_hash: BytesN<32>, data_hash: BytesN<32>) -> Hash<32> {
    let domain_separator: BytesN<32> = env
        .storage()
        .instance()
        .get(&AuthDataKey::DomainSeparator)
        .unwrap();

    let mut msg: Bytes = domain_separator.into();
    msg.extend_from_array(&signer_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());

    // TODO: use an appropriate non tx overlapping prefix
    env.crypto().keccak256(&msg)
}

fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) {
    let minimum_rotation_delay: u64 = env
        .storage()
        .instance()
        .get(&AuthDataKey::MinimumRotationDelay)
        .unwrap();

    let last_rotation_timestamp: u64 = env
        .storage()
        .instance()
        .get(&AuthDataKey::LastRotationTimestamp)
        .unwrap_or(0);

    let current_timestamp = env.ledger().timestamp();

    if enforce_rotation_delay
        && (current_timestamp - last_rotation_timestamp < minimum_rotation_delay)
    {
        panic_with_error!(env, AuthError::InsufficientRotationDelay);
    }

    env.storage()
        .instance()
        .set(&AuthDataKey::LastRotationTimestamp, &current_timestamp);
}

fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
    let mut total_weight = 0u128;

    for ProofSigner {
        signer:
            WeightedSigner {
                signer: public_key,
                weight,
            },
        signature,
    } in proof.signers.iter()
    {
        if let ProofSignature::Signed(signature) = signature {
            env.crypto()
                .ed25519_verify(&public_key, msg_hash.to_bytes().as_ref(), &signature);

            total_weight = total_weight.checked_add(weight).unwrap();

            if total_weight >= proof.threshold {
                return true;
            }
        }
    }

    false
}

/// Check if signer set is valid, i.e signer/pub key hash are in sorted order,
/// weights are non-zero and sum to at least threshold
fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) {
    if weighted_signers.signers.is_empty() {
        panic_with_error!(env, AuthError::InvalidSigners);
    }

    // TODO: what's the min address/hash?
    let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
    let mut total_weight = 0u128;

    for signer in weighted_signers.signers.iter() {
        if previous_signer >= signer.signer {
            panic_with_error!(env, AuthError::InvalidSigners);
        }

        if signer.weight == 0 {
            panic_with_error!(env, AuthError::InvalidWeights);
        }

        previous_signer = signer.signer;
        total_weight = total_weight.checked_add(signer.weight).unwrap();
    }

    let threshold = weighted_signers.threshold;
    if threshold == 0 || total_weight < threshold {
        panic_with_error!(env, AuthError::InvalidThreshold);
    }
}

fn epoch(env: &Env) -> u64 {
    env.storage().instance().get(&AuthDataKey::Epoch).unwrap()
}
