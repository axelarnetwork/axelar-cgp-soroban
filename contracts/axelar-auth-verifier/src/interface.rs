use soroban_sdk::{Address, Bytes, Env};

/// Interface for the Axelar Gateway Auth.
// #[contractclient(crate_path = "crate", name = "AxelarGatewayAuthClient")]
pub trait AxelarGatewayAuthInterface {
    fn validate_proof(env: Env, proof: Bytes) -> bool;

    fn transfer_operatorship(env: Env, caller: Address, new_operator_set: Bytes);
}
