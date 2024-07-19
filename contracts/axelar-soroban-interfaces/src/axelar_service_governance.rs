use soroban_sdk::{contractclient, Address, Bytes, Env, String, Symbol, Val, Vec};

/// Interface for the Axelar Operators contract.
#[contractclient(name = "AxelarServiceGovernance")]
pub trait AxelarServiceGovernanceInterface {
    /// Initialize the operators contract with an owner.
    fn initialize(env: Env, auth_module: Address, gateway: Address, minimum_time_delay: u64);

    /// Execute a function for a GMP call.
    fn execute(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    );

    fn get_proposal_eta(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> u64;

    fn execute_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val;

    fn is_multisig_proposal_approved(
        env: Env,
        target: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> bool;

    fn execute_multisig_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val;
}
