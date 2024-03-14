use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String, U256};

use crate::interface::AxelarGasServiceInterface;
use crate::storage_types::DataKey;
use crate::{error::Error, event};

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
    ) {
        event::native_gas_paid_for_contract_call(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
        )
    }

    fn collect_fees(
        env: Env,
        receiver: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), Error> {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .unwrap();

        gas_collector.require_auth();

        //TODO: sanity check address zero

        if amount == 0 {
            return Err(Error::InvalidAmounts);
        }

        let token_client = token::Client::new(&env, &token_address);

        let contract_token_balance = token_client.balance(&env.current_contract_address());

        if contract_token_balance >= amount {
            token_client.transfer(&env.current_contract_address(), &receiver, &amount)
        } else {
            return Err(Error::InsufficientBalance);
        }

        event::fee_collected(&env, &gas_collector, &token_address, amount);

        Ok(())
    }

    fn refund(
        env: Env,
        tx_hash: BytesN<32>,
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

        event::refunded(&env, tx_hash, log_index, &receiver, &token_addr, amount);

        token::Client::new(&env, &token_addr).transfer(
            &env.current_contract_address(),
            &receiver,
            &amount,
        );
    }
}
