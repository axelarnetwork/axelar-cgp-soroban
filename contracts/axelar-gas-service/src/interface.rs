use crate::error::Error;
use soroban_sdk::{Address, Bytes, BytesN, Env, String, Vec, U256};
/// Interface for the Axelar Gas Service.
// #[contractclient(crate_path = "crate", name = "AxelarGasService")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using native currency for a contract call on a destination chain. This function is called on the source chain before calling the gateway to execute a remote contract.
    fn pay_native_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
    );

    /// Allows the gasCollector to collect accumulated fees from the contract.
    fn collect_fees(
        env: Env,
        receiver: Address,
        token_addr: Vec<Address>,
        amounts: Vec<i128>,
    ) -> Result<(), Error>;

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction. Only callable by the gasCollector.
    fn refund(
        env: Env,
        tx_hash: BytesN<32>,
        log_index: U256,
        receiver: Address,
        token: Address,
        amount: i128,
    );
}
