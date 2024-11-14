#![cfg(test)]
extern crate std;

use axelar_operators::error::ContractError;
use axelar_soroban_std::{
    assert_contract_err, assert_last_emitted_event, testutils::assert_invocation,
};

use axelar_operators::contract::{AxelarOperators, AxelarOperatorsClient};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Env, Symbol, Vec,
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

fn setup_env<'a>() -> (Env, Address, AxelarOperatorsClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let user = Address::generate(&env);
    let contract_id = env.register(AxelarOperators, (&user,));
    let client = AxelarOperatorsClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
fn transfer_owner() {
    let (env, _, client) = setup_env();

    let new_owner = Address::generate(&env);

    // transfer ownership to the new owner
    client.transfer_ownership(&new_owner);

    assert_invocation(
        &env,
        &client.owner(),
        &client.address,
        "transfer_ownership",
        (new_owner.clone(),),
    );

    assert_last_emitted_event(
        &env,
        &client.address,
        (
            Symbol::new(&env, "ownership_transferred"),
            client.owner(),
            new_owner.clone(),
        ),
        (),
    );

    let retrieved_owner = client.owner();
    assert_eq!(retrieved_owner, new_owner);
}

#[test]
fn add_operator() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // set operator as an operator
    client.add_operator(&operator);

    assert_invocation(
        &env,
        &client.owner(),
        &client.address,
        "add_operator",
        (operator.clone(),),
    );

    assert_last_emitted_event(
        &env,
        &client.address,
        (Symbol::new(&env, "operator_added"), operator.clone()),
        (),
    );

    let is_operator_final = client.is_operator(&operator);
    assert!(is_operator_final);
}

#[test]
fn fail_add_operator_duplicate() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // set operator as an operator
    client.add_operator(&operator);

    // set existing operator as an operator, should panic
    assert_contract_err!(
        client.try_add_operator(&operator),
        ContractError::OperatorAlreadyAdded
    );
}

#[test]
fn remove_operator() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    // set operator as an operator
    client.add_operator(&operator);

    let is_operator_initial = client.is_operator(&operator);
    assert!(is_operator_initial);

    // remove operator as an operator
    client.remove_operator(&operator);

    assert_invocation(
        &env,
        &client.owner(),
        &client.address,
        "remove_operator",
        (operator.clone(),),
    );

    assert_last_emitted_event(
        &env,
        &client.address,
        (Symbol::new(&env, "operator_removed"), operator.clone()),
        (),
    );

    let is_operator_final = client.is_operator(&operator);
    assert!(!is_operator_final);
}

#[test]
fn fail_remove_operator_non_existant() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);
    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // remove operator that is not an operator, should panic
    assert_contract_err!(
        client.try_remove_operator(&operator),
        ContractError::NotAnOperator
    );
}

#[test]
fn execute() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute as an operator
    client.execute(
        &operator,
        &client.address,
        &symbol_short!("method"),
        &Vec::new(&env),
    );

    assert_last_emitted_event(&env, &client.address, (symbol_short!("executed"),), ());
}

#[test]
fn fail_execute_not_operator() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute with a non-operator, should panic
    assert_contract_err!(
        client.try_execute(&client.owner(), &client.address, &symbol_short!("method"), &Vec::new(&env)),
        ContractError::NotAnOperator
    );
}

#[test]
#[should_panic(expected = "This method should fail")]
fn fail_execute_when_target_panics() {
    let (env, _, client) = setup_env();

    let operator = Address::generate(&env);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute as an operator
    client.execute(
        &operator,
        &client.address,
        &symbol_short!("failing"),
        &Vec::new(&env),
    );
}
