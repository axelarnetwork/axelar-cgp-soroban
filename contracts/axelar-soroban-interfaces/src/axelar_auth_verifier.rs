use soroban_sdk::{Address, Bytes, BytesN, Env, contractclient};

/// Interface for the Axelar Auth Verifier.
#[contractclient(name = "AxelarAuthVerifierClient")]
pub trait AxelarAuthVerifierInterface {
    fn initialize(env: Env, owner: Address, previous_signer_retention: u32, operator_set: Bytes);

    fn validate_proof(env: &Env, msg_hash: BytesN<32>, proof: Bytes) -> bool;

    fn transfer_operatorship(env: Env, new_operator_set: Bytes);
}
