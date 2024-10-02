#![cfg(test)]
extern crate std;

use axelar_soroban_interfaces::axelar_operators::OperatorError;
use axelar_soroban_std::{assert_emitted_event, testutils::assert_invocation};

use crate::{
    contract::{AxelarOperators, AxelarOperatorsClient},
    storage_types::DataKey,
};
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

fn setup_env<'a>() -> (Env, Address, AxelarOperatorsClient<'a>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarOperators);
    let client = AxelarOperatorsClient::new(&env, &contract_id);

    let target_contract_id = env.register_contract(None, TestTarget);

    (env, contract_id, client, target_contract_id)
}

#[test]
fn test_initialize() {
    let (env, contract_id, client, _) = setup_env();
    let user = Address::generate(&env);

    client.initialize(&user);

    assert!(env.as_contract(&contract_id, || {
        env.storage()
            .instance()
            .get::<DataKey, bool>(&DataKey::Initialized)
            .unwrap()
    }),);

    assert_eq!(
        env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .get::<DataKey, Address>(&DataKey::Owner)
                .unwrap()
        }),
        user
    );
}

#[test]
fn fail_already_initialized() {
    let (env, _, client, _) = setup_env();
    let user = Address::generate(&env);

    client.initialize(&user);

    assert_eq!(
        client.try_initialize(&user),
        Err(Ok(OperatorError::AlreadyInitialized))
    );
}

#[test]
fn transfer_owner() {
    let (env, _, client, _) = setup_env();

    let initial_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    client.initialize(&initial_owner);

    // transfer ownership to the new owner
    client.transfer_ownership(&new_owner);

    assert_invocation(
        &env,
        &initial_owner,
        &client.address,
        "transfer_ownership",
        (new_owner.clone(),),
    );

    assert_emitted_event(
        &env,
        0,
        &client.address,
        (symbol_short!("ownership"), initial_owner, new_owner.clone()),
        (),
    );

    let retrieved_owner = client.owner();
    assert_eq!(retrieved_owner, new_owner);
}

#[test]
fn test_add_operator() {
    let (env, _, client, _) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // set operator as an operator
    client.add_operator(&operator);

    assert_invocation(
        &env,
        &owner,
        &client.address,
        "add_operator",
        (operator.clone(),),
    );

    assert_emitted_event(
        &env,
        0,
        &client.address,
        (symbol_short!("added"), operator.clone()),
        (),
    );

    let is_operator_final = client.is_operator(&operator);
    assert!(is_operator_final);
}

#[test]
fn fail_add_operator_duplicate() {
    let (env, _, client, _) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // set operator as an operator
    client.add_operator(&operator);

    // set existing operator as an operator, should panic
    let res = client.try_add_operator(&operator);
    assert_eq!(res, Err(Ok(OperatorError::OperatorAlreadyAdded)));
}

#[test]
fn test_remove_operator() {
    let (env, _, client, _) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    // set operator as an operator
    client.add_operator(&operator);

    let is_operator_initial = client.is_operator(&operator);
    assert!(is_operator_initial);

    // remove operator as an operator
    client.remove_operator(&operator);

    assert_invocation(
        &env,
        &owner,
        &client.address,
        "remove_operator",
        (operator.clone(),),
    );

    assert_emitted_event(
        &env,
        -1,
        &client.address,
        (symbol_short!("removed"), operator.clone()),
        (),
    );

    let is_operator_final = client.is_operator(&operator);
    assert!(!is_operator_final);
}

#[test]
fn fail_remove_operator_non_existant() {
    let (env, _, client, _) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    let is_operator_initial = client.is_operator(&operator);
    assert!(!is_operator_initial);

    // remove operator that is not an operator, should panic
    let res = client.try_remove_operator(&operator);
    assert_eq!(res, Err(Ok(OperatorError::NotAnOperator)));
}

#[test]
fn test_execute() {
    let (env, _, client, target) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute as an operator
    client.execute(
        &operator,
        &target,
        &symbol_short!("method"),
        &Vec::new(&env),
    );

    assert_emitted_event(&env, -1, &target, (symbol_short!("executed"),), ());
}

#[test]
fn fail_execute_not_operator() {
    let (env, _, client, target) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute with a non-operator, should panic
    assert!(matches!(
        client.try_execute(&owner, &target, &symbol_short!("method"), &Vec::new(&env)),
        Err(Ok(OperatorError::NotAnOperator))
    ));
}

#[test]
#[should_panic(expected = "This method should fail")]
fn fail_execute_when_target_panics() {
    let (env, _, client, target) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    client.initialize(&owner);

    // set operator as an operator
    client.add_operator(&operator);

    // call execute as an operator
    client.execute(
        &operator,
        &target,
        &symbol_short!("failing"),
        &Vec::new(&env),
    );
}
