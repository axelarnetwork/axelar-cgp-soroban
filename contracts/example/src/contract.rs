use crate::event;

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

#[contract]
pub struct Example;

#[contractimpl]
impl Example {
    pub fn __constructor(env: Env, gateway: Address, gas_service: Address) {
        env.storage().instance().set(&DataKey::Gateway, &gateway);
        env.storage()
            .instance()
            .set(&DataKey::GasService, &gas_service);
    }

    pub fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Gateway).unwrap()
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
        gas_token: Address,
        gas_amount: i128,
    ) {
        let gateway = axelar_gateway::Client::new(&env, &Self::gateway(&env));
        let gas_service = axelar_gas_service::Client::new(&env, &Self::gas_service(&env));

        caller.require_auth();

        gas_service.pay_gas_for_contract_call(
            &caller,
            &destination_chain,
            &destination_address,
            &message,
            &caller,
            &gas_token,
            &gas_amount,
        );

        gateway.call_contract(
            &env.current_contract_address(),
            &destination_chain,
            &destination_address,
            &message,
        );
    }

    pub fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    ) {
        let gateway = axelar_gateway::Client::new(&env, &Self::gateway(&env));

        // Validate the contract call was approved by the gateway
        gateway.validate_message(
            &env.current_contract_address(),
            &source_chain,
            &message_id,
            &source_address,
            &env.crypto().keccak256(&payload).into(),
        );

        event::executed(&env, source_chain, message_id, source_address, payload);
    }
}
