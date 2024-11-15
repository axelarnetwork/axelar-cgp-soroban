use axelar_soroban_std::assert_some;
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, Env, String};

use axelar_soroban_std::{ensure, types::Token};

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

    /// Add additional gas payment after initiating a cross-chain call.
    pub fn add_gas(
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

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    pub fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError> {
        let gas_collector: Address =
            assert_some!(env.storage().instance().get(&DataKey::GasCollector));

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

    /// Refunds gas payment to the receiver in relation to a specific cross-chain transaction.
    ///
    /// Only callable by the `gas_collector`.
    pub fn refund(env: Env, message_id: String, receiver: Address, token: Token) {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .expect("gas collector not found");

        gas_collector.require_auth();

        token::Client::new(&env, &token.address).transfer(
            &env.current_contract_address(),
            &receiver,
            &token.amount,
        );

        event::refunded(&env, message_id, receiver, token);
    }
}

#[cfg(test)]
mod tests {
    use axelar_soroban_std::assert_some;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    use super::{AxelarGasService, AxelarGasServiceClient, DataKey};

    #[test]
    fn initialize_gas_service() {
        let env = Env::default();

        let gas_collector = Address::generate(&env);
        let contract_id = env.register(AxelarGasService, (&gas_collector,));
        let _client = AxelarGasServiceClient::new(&env, &contract_id);

        let stored_collector_address = env.as_contract(&contract_id, || {
            assert_some!(env
                .storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::GasCollector))
        });
        assert_eq!(stored_collector_address, gas_collector);
    }
}
