use soroban_sdk::{Address, Bytes, BytesN, Env, String};

use crate::error::Error;

/// Interface for the Axelar Gas Service.
// #[contractclient(crate_path = "crate", name = "AxelarGasService")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using native currency for a contract call on a destination chain. This function is called on the source chain before calling the gateway to execute a remote contract.
    fn pay_native_gas_for_contract_call();
    
    /// Allows the gasCollector to collect accumulated fees from the contract.
    fn collect_fees();
    
    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction. Only callable by the gasCollector.
    fn refund();

}
