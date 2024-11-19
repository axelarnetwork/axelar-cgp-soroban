use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String};

use axelar_soroban_std::ensure;

use crate::error::ContractError;
use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    /// Initialize the gas service contract with a gas_collector address.
    pub fn __constructor(env: Env, gas_collector: Address) {
        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);
    }

    /// Pay for gas using a token for a contract call on a destination chain.
    ///
    /// This function is called on the source chain before calling the gateway to execute a remote contract.
    pub fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token_address: Address,
        token_amount: i128,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        ensure!(token_amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token_address).transfer_from(
            &env.current_contract_address(),
            &sender,
            &env.current_contract_address(),
            &token_amount,
        );

        event::gas_paid_for_contract_call(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
            token_address,
            token_amount,
        );

        Ok(())
    }

    /// Add additional gas payment after initiating a cross-chain call.
    pub fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        token_address: Address,
        token_amount: i128,
        refund_address: Address,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        ensure!(token_amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token_address).transfer_from(
            &env.current_contract_address(),
            &sender,
            &env.current_contract_address(),
            &token_amount,
        );

        event::gas_added(
            &env,
            message_id,
            token_address,
            token_amount,
            refund_address,
        );

        Ok(())
    }

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    pub fn collect_fees(
        env: Env,
        receiver: Address,
        token_address: Address,
        token_amount: i128,
    ) -> Result<(), ContractError> {
        let gas_collector = Self::gas_collector(&env);
        gas_collector.require_auth();

        ensure!(token_amount > 0, ContractError::InvalidAmount);

        let token_client = token::Client::new(&env, &token_address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        ensure!(
            contract_token_balance >= token_amount,
            ContractError::InsufficientBalance
        );
        token_client.transfer(&env.current_contract_address(), &receiver, &token_amount);

        event::fee_collected(&env, gas_collector, token_address, token_amount);

        Ok(())
    }

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction.
    ///
    /// Only callable by the `gas_collector`.
    pub fn refund(
        env: Env,
        message_id: String,
        receiver: Address,
        token_address: Address,
        token_amount: i128,
    ) {
        Self::gas_collector(&env).require_auth();

        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token_amount,
        );

        event::refunded(&env, message_id, receiver, token_address, token_amount);
    }

    pub fn gas_collector(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasCollector)
            .expect("gas collector not found")
    }
}
