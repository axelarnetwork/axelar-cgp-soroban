use soroban_sdk::{
    contract, contractimpl, panic_with_error, token, Address, Bytes, Env, String, U256,
};

use axelar_soroban_std::types::{Hash, TokenDetails};

use crate::storage_types::DataKey;
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_gas_service::AxelarGasServiceInterface;

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    pub fn initialize(env: Env, gas_collector: Address) {
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
        token_details: TokenDetails,
    ) {
        sender.require_auth();

        let TokenDetails { token_addr, amount } = token_details.clone();

        if amount == 0 {
            panic_with_error!(env, Error::InvalidAmount);
        }

        token::Client::new(&env, &token_addr).transfer(
            &sender,
            &env.current_contract_address(),
            &amount,
        );

        event::gas_paid_for_contract_call(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
            token_details,
        );
    }

    fn collect_fees(env: Env, receiver: Address, token_addr: Address, amount: i128) {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .unwrap();

        gas_collector.require_auth();

        if amount == 0 {
            panic_with_error!(env, Error::InvalidAmount);
        }

        let token_client = token::Client::new(&env, &token_addr);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        if contract_token_balance >= amount {
            token_client.transfer(&env.current_contract_address(), &receiver, &amount)
        } else {
            panic_with_error!(env, Error::InsufficientBalance);
        }

        event::fee_collected(&env, &gas_collector, &token_addr, amount);
    }

    fn refund(
        env: Env,
        tx_hash: Hash,
        log_index: U256,
        receiver: Address,
        token_addr: Address,
        amount: i128,
    ) {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .unwrap();

        gas_collector.require_auth();

        token::Client::new(&env, &token_addr).transfer(
            &env.current_contract_address(),
            &receiver,
            &amount,
        );

        event::refunded(&env, tx_hash, log_index, &receiver, &token_addr, amount);
    }
}
