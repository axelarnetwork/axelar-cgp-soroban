use axelar_soroban_interfaces::{
    axelar_gateway::AuthError,
    types::{ProofSignature, ProofSigner, WeightedSigner},
};
use axelar_soroban_std::ensure;
use soroban_sdk::{crypto::Hash, xdr::ToXdr, Bytes, BytesN, Env, Vec};

use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_interfaces::types::{Proof, WeightedSigners};

pub fn initialize_auth(
    env: Env,
    domain_separator: BytesN<32>,
    minimum_rotation_delay: u64,
    previous_signer_retention: u64,
    initial_signers: Vec<WeightedSigners>,
) -> Result<(), AuthError> {
    env.storage().instance().set(&DataKey::Epoch, &0_u64);

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

    ensure!(!initial_signers.is_empty(), AuthError::InvalidSigners);

    for signers in initial_signers.into_iter() {
        rotate_signers(&env, &signers, false)?;
    }

    Ok(())
}

pub fn validate_proof(env: &Env, data_hash: BytesN<32>, proof: Proof) -> Result<bool, AuthError> {
    let signers_set = proof.weighted_signers();

    let signers_hash: BytesN<32> = env.crypto().keccak256(&signers_set.to_xdr(env)).into();

    let signers_epoch: u64 = env
        .storage()
        .persistent()
        .get(&DataKey::EpochBySignerHash(signers_hash.clone()))
        .unwrap_or(0);

    ensure!(signers_epoch != 0, AuthError::InvalidSigners);

    let current_epoch: u64 = epoch(env)?;

    let is_latest_signers: bool = signers_epoch == current_epoch;

    let previous_signers_retention: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PreviousSignerRetention)
        .ok_or(AuthError::NotInitialized)?;

    ensure!(
        current_epoch - signers_epoch <= previous_signers_retention,
        AuthError::InvalidSigners
    );

    let msg_hash = message_hash_to_sign(env, signers_hash, data_hash);

    ensure!(
        validate_signatures(env, msg_hash, proof),
        AuthError::InvalidSignatures
    );
    Ok(is_latest_signers)
}

pub fn rotate_signers(
    env: &Env,
    new_signers: &WeightedSigners,
    enforce_rotation_delay: bool,
) -> Result<(), AuthError> {
    validate_signers(env, new_signers)?;

    update_rotation_timestamp(env, enforce_rotation_delay)?;

    let new_signers_hash: BytesN<32> = env
        .crypto()
        .keccak256(&new_signers.clone().to_xdr(env))
        .into();
    let new_epoch: u64 = epoch(env)? + 1;

    env.storage().instance().set(&DataKey::Epoch, &new_epoch);

    env.storage()
        .persistent()
        .set(&DataKey::SignerHashByEpoch(new_epoch), &new_signers_hash);

    // If new_signers has been rotated to before, we will overwrite the epoch to point to the latest
    env.storage().persistent().set(
        &DataKey::EpochBySignerHash(new_signers_hash.clone()),
        &new_epoch,
    );

    event::rotate_signers(env, new_signers.clone());
    Ok(())
}

fn message_hash_to_sign(env: &Env, signers_hash: BytesN<32>, data_hash: BytesN<32>) -> Hash<32> {
    let domain_separator: BytesN<32> = env
        .storage()
        .instance()
        .get(&DataKey::DomainSeparator)
        .unwrap();

    let mut msg: Bytes = domain_separator.into();
    msg.extend_from_array(&signers_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());

    // TODO: use an appropriate non tx overlapping prefix
    env.crypto().keccak256(&msg)
}

fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) -> Result<(), AuthError> {
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

    ensure!(
        !enforce_rotation_delay
            || (current_timestamp - last_rotation_timestamp >= minimum_rotation_delay),
        AuthError::InsufficientRotationDelay
    );

    env.storage()
        .instance()
        .set(&DataKey::LastRotationTimestamp, &current_timestamp);
    Ok(())
}

fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
    let mut total_weight = 0u128;

    for ProofSigner {
        signer: WeightedSigner {
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
fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) -> Result<(), AuthError> {
    ensure!(
        !weighted_signers.signers.is_empty(),
        AuthError::InvalidSigners
    );

    // TODO: what's the min address/hash?
    let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
    let mut total_weight = 0u128;

    for signer in weighted_signers.signers.iter() {
        ensure!(previous_signer < signer.signer, AuthError::InvalidSigners);

        ensure!(signer.weight != 0, AuthError::InvalidWeights);

        previous_signer = signer.signer;
        total_weight = total_weight.checked_add(signer.weight).unwrap();
    }

    let threshold = weighted_signers.threshold;
    ensure!(
        threshold != 0 && total_weight >= threshold,
        AuthError::InvalidThreshold
    );
    Ok(())
}

fn epoch(env: &Env) -> Result<u64, AuthError> {
    env.storage()
        .instance()
        .get(&DataKey::Epoch)
        .ok_or(AuthError::NotInitialized)
}
