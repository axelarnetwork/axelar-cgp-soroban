use soroban_sdk::{contractclient, Address, Env, Vec};

use axelar_soroban_std::types::Hash;

use crate::types::{Proof, WeightedSigners};

/// Interface for the Axelar Auth Verifier.
#[contractclient(name = "AxelarAuthVerifierClient")]
pub trait AxelarAuthVerifierInterface {
    fn initialize(env: Env, owner: Address, previous_signer_retention: u32, domain_separator: Hash, minimum_rotation_delay: u64, initial_signers: Vec<WeightedSigners>);

    fn epoch(env: Env) -> u64;

    fn validate_proof(env: Env, data_hash: Hash, proof: Proof) -> bool;

    fn rotate_signers(env: Env, new_signers: WeightedSigners, enforce_rotation_delay: bool);
}
