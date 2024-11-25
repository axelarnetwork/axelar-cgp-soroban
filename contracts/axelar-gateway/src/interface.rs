use crate::{
    error::ContractError,
    types::{Message, Proof, WeightedSigners},
    AxelarGatewayMessagingInterface,
};
use axelar_soroban_std::shared_interfaces::UpgradeableInterface;
use soroban_sdk::{contractclient, Address, BytesN, Env, Vec};

#[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface: AxelarGatewayMessagingInterface + UpgradeableInterface {
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

    /// Returns the epoch by signers hash.
    fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError>;

    /// Returns the signers hash by epoch.
    fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError>;
}
