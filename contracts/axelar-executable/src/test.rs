#![cfg(test)]
extern crate std;

use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, testutils::Address as _, Address,
    Bytes, BytesN, Env, String,
};

use axelar_soroban_std::testutils::assert_emitted_event;

use axelar_gateway::contract::AxelarGatewayClient;

use crate::{error::Error, AxelarExecutableInterface};
use crate::interface::AxelarExecutableInternal;

#[contract]
pub struct AxelarExecutableTest;

impl AxelarExecutableInternal for AxelarExecutableTest {
    fn execute_internal(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) {
        env.events().publish((symbol_short!("executed"),), ());
    }
}

#[contractimpl]
impl AxelarExecutableInterface for AxelarExecutableTest {
    type Internal = AxelarExecutableTest;

    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&"gateway").unwrap()
    }
}

#[contractimpl]
impl AxelarExecutableTest {
    pub fn initialize(env: Env, gateway: Address) {
        env.storage().instance().set(&"initialized", &true);

        env.storage().instance().set(&"gateway", &gateway);
    }
}

#[contract]
pub struct MockAxelarGateway;

#[contractimpl]
impl MockAxelarGateway {
    pub fn validate_contract_call(
        _env: Env,
        _caller: Address,
        _command_id: soroban_sdk::BytesN<32>,
        _source_chain: soroban_sdk::String,
        _source_address: soroban_sdk::String,
        _payload_hash: soroban_sdk::BytesN<32>,
    ) -> bool {
        true
    }
}

fn setup_env<'a>() -> (Env, Address, AxelarExecutableTestClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarExecutableTest);
    let client = AxelarExecutableTestClient::new(&env, &contract_id);

    (env, contract_id, client)
}

fn initialize(env: &Env, client: &AxelarExecutableTestClient) {
    let gateway_contract_id = env.register_contract(None, MockAxelarGateway);

    client.initialize(&gateway_contract_id);
}

#[test]
fn test_initialize() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    client.initialize(&user);

    let _ = client.gateway();
}

#[test]
fn test_execute() {
    let (env, _, client) = setup_env();

    initialize(&env, &client);

    let payload = Bytes::from_array(&env, &[1, 2, 3]);

    let (
        axelar_gateway::types::ContractCallApproval {
            source_chain,
            source_address,
            contract_address: _,
            payload_hash,
        },
        _,
    ) = axelar_gateway::testutils::generate_test_approval(&env);

    let command_id = payload_hash;

    client.execute(&command_id, &source_chain, &source_address, &payload);

    assert_emitted_event(&env, -1, &client.address, (symbol_short!("executed"),), ());
}
