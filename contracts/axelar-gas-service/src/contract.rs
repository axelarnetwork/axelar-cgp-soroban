use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, String};

use crate::error::ContractError;
use crate::event;
use crate::interface::AxelarGasServiceInterface;
use crate::storage_types::DataKey;
use axelar_soroban_std::shared_interfaces::OwnershipInterface;
use axelar_soroban_std::shared_interfaces::{migrate, UpgradeableInterface};
use axelar_soroban_std::{ensure, shared_interfaces, types::Token};

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    /// Initialize the gas service contract with a gas_collector address.
    pub fn __constructor(env: Env, owner: Address, gas_collector: Address) {
        shared_interfaces::set_owner(&env, &owner);
        env.storage()
            .instance()
            .set(&DataKey::GasCollector, &gas_collector);
    }

    pub fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::MigrationNotAllowed)
    }
}

impl AxelarGasService {
    // Modify this function to add migration logic
    #[allow(clippy::missing_const_for_fn)] // exclude no-op implementations from this lint
    fn run_migration(_env: &Env, _migration_data: ()) {}
}

#[contractimpl]
impl UpgradeableInterface for AxelarGasService {
    fn version(env: &Env) -> String {
        String::from_str(env, env!("CARGO_PKG_VERSION"))
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        shared_interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contractimpl]
impl OwnershipInterface for AxelarGasService {
    // boilerplate necessary for the contractimpl macro to include function in the generated client
    fn owner(env: &Env) -> Address {
        shared_interfaces::owner(env)
    }
}

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    fn pay_gas(
        env: Env,
        sender: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
        spender: Address,
        token: Token,
        metadata: Bytes,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer(
            &spender,
            &env.current_contract_address(),
            &token.amount,
        );

        event::gas_paid(
            &env,
            sender,
            destination_chain,
            destination_address,
            payload,
            spender,
            token,
            metadata,
        );

        Ok(())
    }

    fn add_gas(
        env: Env,
        sender: Address,
        message_id: String,
        spender: Address,
        token: Token,
    ) -> Result<(), ContractError> {
        spender.require_auth();

        ensure!(token.amount > 0, ContractError::InvalidAmount);

        token::Client::new(&env, &token.address).transfer(
            &spender,
            &env.current_contract_address(),
            &token.amount,
        );

        event::gas_added(&env, sender, message_id, spender, token);

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
