use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::error::ContractError;

#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface {
    #[allow(clippy::too_many_arguments)]
    /// Pay for gas using a token for sending a message on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to send a message.
    ///
    /// # Notes
    /// - The `sender` is distinct from the `spender`. The `sender` initiates the action
    ///   requiring gas payment but does not directly pay for the gas. Instead, the
    ///   `spender` is responsible for authorizing and funding the payment.
    /// - The `spender` can also serve as the `refund_address`, receiving any unused gas
    ///   or reimbursement as applicable.
    fn pay_gas(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Token,
        metadata: Bytes,
    ) -> Result<(), ContractError>;

    /// Add additional gas payment after initiating a cross-chain message.
    /// # Notes
    /// - The `sender` is distinct from the `spender`. The `sender` initiates the action
    ///   requiring gas payment but does not directly pay for the gas. Instead, the
    ///   `spender` is responsible for authorizing and funding the payment.
    /// - The `spender` can also serve as the `refund_address`, receiving any unused gas
    ///   or reimbursement as applicable.
    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        spender: Address,
        token: Token,
    ) -> Result<(), ContractError>;

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError>;

    /// Refunds gas payment to the receiver in relation to a specific cross-chain message.
    ///
    /// Only callable by the `gas_collector`.
    fn refund(env: Env, message_id: String, receiver: Address, token: Token);

    /// Returns the address of the `gas_collector`.
    fn gas_collector(env: &Env) -> Address;
}
