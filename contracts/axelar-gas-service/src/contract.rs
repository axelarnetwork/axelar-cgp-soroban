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
    pub fn initialize(env: Env, gas_collector: Address) -> Result<(), ContractError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            ContractError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);

        Ok(())
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

    /// Allows the `gas_collector` to collect accumulated fees from the contract.
    ///
    /// Only callable by the `gas_collector`.
    pub fn collect_fees(env: Env, receiver: Address, token: Token) -> Result<(), ContractError> {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .ok_or(ContractError::NotInitialized)?;

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
    pub fn refund(
        env: Env,
        message_id: String,
        receiver: Address,
        token: Token,
    ) -> Result<(), ContractError> {
        let gas_collector: Address = env
            .storage()
            .instance()
            .get(&DataKey::GasCollector)
            .ok_or(ContractError::NotInitialized)?;

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

#[cfg(test)]
mod tests {
    use axelar_soroban_std::assert_some;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, Env};

    use super::{AxelarGasService, AxelarGasServiceClient, DataKey};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, AxelarGasService);
        let client = AxelarGasServiceClient::new(&env, &contract_id);
        let gas_collector = Address::generate(&env);
        client.initialize(&gas_collector);

        assert!(assert_some!(env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
        })));

        let stored_collector_address = assert_some!(env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::GasCollector)
        }));
        assert_eq!(stored_collector_address, gas_collector);
    }
}
