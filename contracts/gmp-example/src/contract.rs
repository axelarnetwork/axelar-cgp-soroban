use crate::{error::ContractError, event};
use axelar_gas_service::AxelarGasServiceClient;
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::{ensure, types::Token};
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String};

use crate::storage_types::DataKey;

use axelar_gateway::executable::AxelarExecutableInterface;

#[contract]
pub struct GmpExample;

#[contractimpl]
impl AxelarExecutableInterface for GmpExample {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Gateway).unwrap()
    }

    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) {
        Self::validate(
            env.clone(),
            source_chain.clone(),
            message_id.clone(),
            source_address.clone(),
            payload.clone(),
        );

        event::executed(&env, source_chain, message_id, source_address, payload);
    }
}

#[contractimpl]
impl GmpExample {
    pub fn initialize_gmp_example(
        env: Env,
        gateway: Address,
        gas_service: Address,
    ) -> Result<(), ContractError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            ContractError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);

        Ok(())
    }

    fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    pub fn send(
        env: Env,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        token: Token,
    ) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));
        let gas_service = AxelarGasServiceClient::new(&env, &Self::gas_service(&env));

        gas_service.pay_gas_for_contract_call(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
            &env.current_contract_address(),
            &token,
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }
}
