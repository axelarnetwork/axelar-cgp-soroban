#![cfg(test)]
extern crate std;

use axelar_soroban_std::{assert_emitted_event, testutils::assert_invocation};

use crate::contract::{AxelarServiceGovernance, AxelarServiceGovernanceClient};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Env, Vec,
};

#[contract]
pub struct TestTarget;

#[contractimpl]
impl TestTarget {
    pub fn method(env: Env) {
        env.events().publish((symbol_short!("executed"),), ());
    }

    pub fn failing(_env: Env) {
        panic!("This method should fail");
    }
}

fn setup_env<'a>() -> (Env, Address, AxelarServiceGovernanceClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarServiceGovernance);
    let client = AxelarServiceGovernanceClient::new(&env, &contract_id);

    let target_contract_id = env.register_contract(None, TestTarget);

    (env, contract_id, client, target_contract_id)
}

#[test]
fn test_initialize() {
    let (env, _, client, _) = setup_env();
    let user = Address::generate(&env);

    client.initialize(&user);
}

#[test]
#[should_panic(expected = "Already initialized")]
fn fail_already_initialized() {
    let (env, _, client, _) = setup_env();
    let user = Address::generate(&env);

    client.initialize(&user);

    client.initialize(&user);
}

