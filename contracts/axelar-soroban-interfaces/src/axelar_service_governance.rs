use soroban_sdk::{contractclient, Address, Bytes, Env, String, Symbol, Val, Vec};

use axelar_soroban_std::types::Hash;

use crate::types::Proof;

/// Interface for the Axelar Operators contract.
#[contractclient(name = "AxelarServiceGovernance")]
pub trait AxelarServiceGovernanceInterface {
    /// Initialize the operators contract with an owner.
    fn initialize(env: Env, auth_module: Address, gateway: Address);

    /// Execute a function for a GMP call.
    fn execute(
        env: Env,
        command_id: Hash,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    );
    fn execute_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val;

    fn execute_multisig_proposal(
        env: Env,
        target: Address,
        func: Symbol,
        args: Vec<Val>,
        proof: Proof,
    ) -> Val;
}
