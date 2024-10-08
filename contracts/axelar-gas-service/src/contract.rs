use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Bytes, Env, String};

use axelar_soroban_std::{ensure, types::Token};

use crate::event;
use crate::storage_types::DataKey;
use axelar_soroban_interfaces::axelar_gas_service::{AxelarGasServiceInterface, GasServiceError};

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    fn initialize(env: Env, gas_collector: Address) -> Result<(), GasServiceError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            GasServiceError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);

        Ok(())
    }

    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token: Token,
    ) -> Result<(), GasServiceError> {
        sender.require_auth();

        ensure!(token.amount > 0, GasServiceError::InvalidAmount);

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

    fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), GasServiceError> {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .ok_or(GasServiceError::NotInitialized)?;

        gas_collector.require_auth();

        ensure!(token.amount > 0, GasServiceError::InvalidAmount);

        let token_client = token::Client::new(&env, &token.address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        ensure!(
            contract_token_balance >= token.amount,
            GasServiceError::InsufficientBalance
        );
        token_client.transfer(&env.current_contract_address(), &receiver, &token.amount);

        event::fee_collected(&env, gas_collector, token);

        Ok(())
    }

    fn refund(
        env: Env,
        message_id: String,
        receiver: Address,
        token: Token,
    ) -> Result<(), GasServiceError> {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .ok_or(GasServiceError::NotInitialized)?;

        gas_collector.require_auth();

        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        event::refunded(&env, message_id, receiver, token);

        Ok(())
    }
}
