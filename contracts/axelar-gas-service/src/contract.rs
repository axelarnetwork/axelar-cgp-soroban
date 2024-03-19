use soroban_sdk::{contract, contractimpl, panic_with_error, token, Address, Bytes, Env, String};

use axelar_soroban_std::types::Token;

use crate::storage_types::DataKey;
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_gas_service::AxelarGasServiceInterface;

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    fn initialize(env: Env, gas_collector: Address) {
        if env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
        {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);
    }

    fn pay_gas_for_contract_call(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        refund_address: Address,
        token: Token,
    ) {
        sender.require_auth();

        if token.amount <= 0 {
            panic_with_error!(env, Error::InvalidAmount);
        }

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
    }

    fn collect_fees(env: Env, receiver: Address, token: Token) {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .unwrap();

        gas_collector.require_auth();

        if token.amount <= 0 {
            panic_with_error!(env, Error::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &token.address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        if contract_token_balance >= token.amount {
            token_client.transfer(&env.current_contract_address(), &receiver, &token.amount)
        } else {
            panic_with_error!(env, Error::InsufficientBalance);
        }

        event::fee_collected(&env, gas_collector, token);
    }

    fn refund(env: Env, message_id: String, receiver: Address, token: Token) {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .unwrap();

        gas_collector.require_auth();

        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        event::refunded(&env, message_id, receiver, token);
    }
}
