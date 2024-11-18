use crate::event;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contract, contractimpl, Address, Bytes, Env, String};

use crate::storage_types::DataKey;

mod axelar_gateway {
    soroban_sdk::contractimport!(
        file =
            "../../../axelar-cgp-soroban/target/wasm32-unknown-unknown/release/axelar_gateway.wasm"
    );
}

mod axelar_gas_service {
    soroban_sdk::contractimport!(
        file =
            "../../../axelar-cgp-soroban/target/wasm32-unknown-unknown/release/axelar_gas_service.wasm"
    );
}

use ::axelar_gas_service::AxelarGasServiceClient;
use ::axelar_gateway::executable::AxelarExecutableInterface;
use ::axelar_gateway::AxelarGatewayClient;

#[contract]
pub struct Example;

#[contractimpl]
impl AxelarExecutableInterface for Example {
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
        let _ = Self::validate(&env, &source_chain, &message_id, &source_address, &payload);

        event::executed(&env, source_chain, message_id, source_address, payload);
    }
}

#[contractimpl]
impl Example {
    pub fn __constructor(env: Env, gateway: Address, gas_service: Address) {
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
    }

    pub fn gas_service(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::GasService).unwrap()
    }

    pub fn send(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        message: Bytes,
        gas_token: Token,
    ) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));
        let gas_service = AxelarGasServiceClient::new(&env, &Self::gas_service(&env));

        caller.require_auth();

        gas_service.pay_gas_for_contract_call(
            &caller,
            &destination_chain,
            &destination_address,
            &message,
            &caller,
            &gas_token,
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }
}
