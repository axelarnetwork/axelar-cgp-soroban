use axelar_soroban_std::types::{Hash, TokenDetails};
use soroban_sdk::{Address, Bytes, Env, String, U256};

/// Interface for the Axelar Gas Service.
// #[contractclient(crate_path = "crate", name = "AxelarGasService")]
pub trait AxelarGasServiceInterface {
    /// Pay for gas using a token for a contract call on a destination chain. This function is called on the source chain before calling the gateway to execute a remote contract.
    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token_details: TokenDetails,
    );

    /// Allows the gasCollector to collect accumulated fees from the contract.
    fn collect_fees(env: Env, receiver: Address, token_addr: Address, amounts: i128);

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction. Only callable by the gasCollector.
    fn refund(
        env: Env,
        tx_hash: Hash,
        log_index: U256,
        receiver: Address,
        token: Address,
        amount: i128,
    );
}
