use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String, Vec};

use crate::types::{Message, Proof, WeightedSigners};

use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GatewayError {
    Uninitialized = 1,
    EmptyMessages = 2,
    RotationAlreadyExecuted = 3,
    NotLatestSigners = 4,
    InvalidOperators = 5,
    NotInitialized = 6,
    AlreadyInitialized = 7,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    InvalidThreshold = 1,
    DuplicateOperators = 2,
    MalformedSigners = 3,
    LowSignaturesWeight = 4,
    InvalidProof = 5,
    InvalidSigners = 6,
    InsufficientRotationDelay = 7,
    InvalidSignatures = 8,
    InvalidWeights = 9,
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

    fn approve_messages(env: Env, messages: Vec<Message>, proof: Proof) -> Result<(), GatewayError>;

    /// Validate if a contract call with the given payload BytesN<32> and source caller info is approved,
    /// preventing re-validation (i.e distinct contract calls can be validated at most once).
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info is approved.
    fn is_message_approved(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info has been executed.
    fn is_message_executed(env: Env, message_id: String, source_chain: String) -> bool;

    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        enforce_rotation_delay: bool,
    ) -> Result<(), GatewayError>;

    fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), GatewayError>;

    fn operator(env: &Env) -> Result<Address, GatewayError>;
}
