use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

/// Interface for the Axelar Gateway.
#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface {
    /// Initialize the gateway with the given auth module address.
    fn initialize(env: Env, auth_module: Address);

    /// Call a contract on another chain with the given payload. The destination address can validate the contract call on the destination gateway.
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    );

    /// Validate if a contract call with the given payload hash and source caller info is approved,
    /// preventing re-validation (i.e distinct contract calls can be validated at most once).
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_contract_call(
        env: Env,
        caller: Address,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload hash and source caller info is approved.
    fn is_contract_call_approved(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Approve a batch of commands signed by Axelar verifiers, consisting of contract call approvals, and verifier set updates.
    fn execute(env: Env, batch: Bytes);
}
