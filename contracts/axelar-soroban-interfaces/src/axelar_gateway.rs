use soroban_sdk::{contractclient, Address, Bytes, Env, String, Vec};

use crate::types::{Message, Proof, WeightedSigners};
use axelar_soroban_std::types::Hash;

/// Interface for the Axelar Gateway.
#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface {
    /// Initialize the gateway with the given auth module address.
    fn initialize(env: Env, auth_module: Address, operator: Address);

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
    fn validate_message(
        env: Env,
        caller: Address,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload_hash: Hash,
    ) -> bool;

    /// Return true if a contract call with the given payload hash and source caller info is approved.
    fn is_message_approved(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: Hash,
    ) -> bool;

    /// Return true if a contract call with the given payload hash and source caller info has been executed.
    fn is_message_executed(env: Env, message_id: String, source_chain: String) -> bool;

    fn approve_messages(env: Env, messages: Vec<Message>, proof: Proof);

    fn rotate_signers(env: Env, signers: WeightedSigners, proof: Proof);
}
