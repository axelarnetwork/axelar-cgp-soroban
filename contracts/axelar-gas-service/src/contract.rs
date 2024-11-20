use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String};

use axelar_soroban_std::{ensure, types::Token};

use crate::error::ContractError;
use crate::event;
use crate::interface::AxelarGasServiceInterface;
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
}

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token: Token,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer_from(
            &env.current_contract_address(),
            &sender,
            &env.current_contract_address(),
            &token.amount,
        );

        event::gas_paid_for_contract_call(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
            token,
        );

        Ok(())
    }

    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        token: Token,
        refund_address: Address,
    ) -> Result<(), ContractError> {
        sender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer_from(
            &env.current_contract_address(),
            &sender,
            &env.current_contract_address(),
            &token.amount,
        );

        event::gas_added(&env, message_id, token, refund_address);

        Ok(())
    }

    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError> {
        let gas_collector = Self::gas_collector(&env);
        gas_collector.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        let token_client = token::Client::new(&env, &token.address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        ensure!(
            contract_token_balance >= token.amount,
            ContractError::InsufficientBalance
        );
        token_client.transfer(&env.current_contract_address(), &receiver, &token.amount);

        event::fee_collected(&env, gas_collector, token);

        Ok(())
    }

    fn refund(env: Env, message_id: String, receiver: Address, token: Token) {
        Self::gas_collector(&env).require_auth();

        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        event::refunded(&env, message_id, receiver, token);
    }

    fn gas_collector(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::GasCollector)
            .expect("gas collector not found")
    }
}
