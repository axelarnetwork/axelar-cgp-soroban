use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String, Vec};

use crate::types::{Message, Proof, WeightedSigners};

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GatewayError {
    // General
    NotInitialized = 1,
    AlreadyInitialized = 2,
    // Auth
    InvalidThreshold = 3,
    InvalidProof = 4,
    InvalidSigners = 5,
    InsufficientRotationDelay = 6,
    InvalidSignatures = 7,
    InvalidWeight = 8,
    WeightOverflow = 9,
    NotLatestSigners = 11,
    DuplicateSigners = 12,
    // Messages
    EmptyMessages = 13,
}

/// Interface for the Axelar Gateway.
#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface {
    /// Initialize the gateway
    fn initialize(
        env: Env,
        operator: Address,
        domain_separator: BytesN<32>,
        previous_signers_retention: u64,
        minimum_rotation_delay: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), GatewayError>;

    /// Call a contract on another chain with the given payload. The destination address can validate the contract call on the destination gateway.
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    );

    fn approve_messages(env: Env, messages: Vec<Message>, proof: Proof)
        -> Result<(), GatewayError>;

    /// Validate if a contract call with the given payload BytesN<32> and source caller info is approved,
    /// preventing re-validation (i.e distinct contract calls can be validated at most once).
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info is approved.
    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info has been executed.
    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool;

    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        enforce_rotation_delay: bool,
    ) -> Result<(), GatewayError>;

    fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), GatewayError>;

    fn operator(env: &Env) -> Result<Address, GatewayError>;

    fn epoch(env: &Env) -> Result<u64, GatewayError>;

    fn version(env: Env) -> String;

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), GatewayError>;
}
