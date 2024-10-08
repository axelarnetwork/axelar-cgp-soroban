use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, contracterror, Address, Bytes, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GasServiceError {
    InvalidAddress = 1,
    InvalidAmount = 2,
    InsufficientBalance = 3,
    AlreadyInitialized = 4,
    NotInitialized = 5,
}

/// Interface for the Axelar Gas Service.
#[contractclient(name = "AxelarGasServiceClient")]
pub trait AxelarGasServiceInterface {
    /// Initialize the gas service contract with a gas_collector address.
    fn initialize(env: Env, gas_collector: Address) -> Result<(), GasServiceError>;

    /// Pay for gas using a token for a contract call on a destination chain. This function is called on the source chain before calling the gateway to execute a remote contract.
    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token: Token,
    ) -> Result<(), GasServiceError>;

    /// Allows the `gas_collector` to collect accumulated fees from the contract. Only callable by the gas_collector.
    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), GasServiceError>;

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction. Only callable by the gas_collector.
    fn refund(
        env: Env,
        message_id: String,
        receiver: Address,
        token: Token,
    ) -> Result<(), GasServiceError>;
}
