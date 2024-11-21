use crate::{
    error::ContractError,
    types::{Message, Proof, WeightedSigners},
};
use axelar_soroban_std::UpgradeableInterface;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String, Vec};

#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface: UpgradeableInterface {
    /// Sends a message to the specified destination chain and contarct address with a given payload.
    ///
    /// This function is the entry point for general message passing between chains.
    ///
    /// A registered chain name on Axelar must be used for `destination_chain`.
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    );

    /// Checks if a message is approved
    ///
    /// Determines whether a given message, identified by its `source_chain` and `message_id`, is approved.
    ///
    /// Returns true if a message with the given `payload_hash`  is approved.
    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Checks if a message is executed.
    ///
    /// Returns true if the message is executed, false otherwise.
    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool;

    /// Validates if a message is approved. If message was in approved status, status is updated to executed to avoid
    /// replay.
    ///
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Approves a collection of messages.
    fn approve_messages(
        env: Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), ContractError>;

    // TODO: add docstring about how bypass_rotation_delay supposed to be used.
    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        bypass_rotation_delay: bool,
    ) -> Result<(), ContractError>;

    /// Transfers operatorship of the gateway to a new address.
    fn transfer_operatorship(env: Env, new_operator: Address);

    /// Returns the operator address of the gateway.
    fn operator(env: &Env) -> Address;

    /// Returns the epoch of the gateway.
    fn epoch(env: &Env) -> u64;

    /// Transfers ownership of the gateway to a new address.
    fn transfer_ownership(env: Env, new_owner: Address);

    /// Returns the owner address of the gateway.
    fn owner(env: &Env) -> Address;

    /// Returns the epoch by signers hash.
    fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError>;

    /// Returns the signers hash by epoch.
    fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError>;
}
