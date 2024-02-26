use soroban_sdk::{Bytes, BytesN, Env};

/// Interface for the Axelar Auth Verifier.
// #[contractclient(crate_path = "crate", name = "AxelarAuthVerifierClient")]
pub trait AxelarAuthVerifierInterface {
    fn validate_proof(env: &Env, msg_hash: BytesN<32>, proof: Bytes) -> bool;

    fn transfer_operatorship(env: Env, new_operator_set: Bytes);
}
