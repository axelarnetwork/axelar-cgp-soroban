#![cfg(test)]
extern crate std;

use crate::testutils::{
    generate_proof, generate_signers_set, generate_test_message, get_approve_hash, initialize,
    randint,
};
use crate::{contract::AxelarGateway, contract::AxelarGatewayClient};
use axelar_soroban_std::{assert_contract_err, assert_invocation, assert_last_emitted_event};
use soroban_sdk::testutils::BytesN as _;

use crate::types::{ContractError, Message};
use soroban_sdk::Symbol;
use soroban_sdk::{
    bytes,
    testutils::{Address as _, Events, MockAuth, MockAuthInvoke},
    vec, Address, BytesN, Env, IntoVal, String,
};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
fn fails_if_already_initialized() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    // doing the process from testutils::initialize() manually so we can
    // use try_initialize for the second call
    let num_signers = randint(1, 10);
    let previous_signers_retention = 1;

    let signer_set = generate_signers_set(&env, num_signers, BytesN::random(&env));
    let initial_signers = vec![&env, signer_set.signers.clone()];
    let minimum_rotation_delay = 0;

    client.initialize(
        &owner,
        &operator,
        &signer_set.domain_separator,
        &minimum_rotation_delay,
        &(previous_signers_retention as u64),
        &initial_signers,
    );

    assert_contract_err!(
        client.try_initialize(
            &owner,
            &operator,
            &signer_set.domain_separator,
            &minimum_rotation_delay,
            &(previous_signers_retention as u64),
            &initial_signers,
        ),
        ContractError::AlreadyInitialized
    );
}

#[test]
/// Two functions in the gateway contract require initialization:
/// rotate_signers when bypass_rotation_delay = true
/// transfer_operatorship
fn fail_if_not_initialized() {
    let (env, _contract_id, client) = setup_env();
    let new_operator = Address::generate(&env);

    assert_contract_err!(
        client.try_transfer_operatorship(&new_operator),
        ContractError::NotInitialized
    );

    let num_signers = randint(1, 10);
    let signers = generate_signers_set(&env, num_signers, BytesN::random(&env));

    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers);

    let bypass_rotation_delay = true;
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay),
        ContractError::NotInitialized
    );
}

#[test]
fn call_contract() {
    let (env, contract_id, client) = setup_env();

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    client.call_contract(&user, &destination_chain, &destination_address, &payload);

    assert_invocation(
        &env,
        &user,
        &contract_id,
        "call_contract",
        (
            &user,
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
        ),
    );

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "contract_called"),
            user,
            destination_chain,
            destination_address,
            env.crypto().keccak256(&payload),
        ),
        payload,
    );
}

#[test]
fn validate_message() {
    let (env, contract_id, client) = setup_env();

    let (
        Message {
            source_chain,
            message_id,
            source_address,
            contract_address,
            payload_hash,
        },
        _,
    ) = generate_test_message(&env);

    let approved = client.validate_message(
        &contract_address,
        &source_chain,
        &message_id,
        &source_address,
        &payload_hash,
    );
    assert!(!approved);

    assert_invocation(
        &env,
        &contract_address,
        &contract_id,
        "validate_message",
        (
            &contract_address,
            source_chain.clone(),
            message_id.clone(),
            source_address.clone(),
            payload_hash.clone(),
        ),
    );

    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_message() {
    let (env, contract_id, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let Message {
        source_chain,
        message_id,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator, 1, randint(1, 10));

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (Symbol::new(&env, "message_approved"), message.clone()),
        (),
    );

    let is_approved = client.is_message_approved(
        &source_chain,
        &message_id,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(is_approved);

    let approved = client.validate_message(
        &contract_address,
        &source_chain,
        &message_id,
        &source_address,
        &payload_hash,
    );
    assert!(approved);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (Symbol::new(&env, "message_executed"), message.clone()),
        (),
    );

    let is_approved = client.is_message_approved(
        &source_chain,
        &message_id,
        &source_address,
        &contract_address,
        &payload_hash,
    );
    assert!(!is_approved);

    let is_executed = client.is_message_executed(&source_chain, &message_id);
    assert!(is_executed);
}

#[test]
fn fail_execute_invalid_proof() {
    let (env, _contract_id, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator, 1, randint(1, 10));

    let invalid_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, invalid_signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSigners
    );
}

#[test]
fn approve_messages_fail_empty_messages() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(&env, &client, owner, operator, 1, randint(1, 10));

    let messages = soroban_sdk::Vec::new(&env);
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::EmptyMessages
    );
}

#[test]
fn approve_messages_skip_duplicate_message() {
    let (env, _, client) = setup_env();
    let (message, _) = generate_test_message(&env);
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(&env, &client, owner, operator, 1, randint(1, 10));

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    // should not throw an error, should just skip
    let res = client.try_approve_messages(&messages, &proof);
    assert!(res.is_ok());

    // should not emit any more events (2 total because of rotate signers in auth)
    assert_eq!(env.events().all().len(), 2);
}

#[test]
fn rotate_signers() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator, 1, 5);
    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers);
    let bypass_rotation_delay = false;
    let new_epoch: u64 = client.epoch() + 1;

    client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "signers_rotated"),
            new_epoch,
            new_signers.signers.hash(&env),
        ),
        (),
    );

    // test approve with new signer set
    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, new_signers);
    client.approve_messages(&messages, &proof);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (Symbol::new(&env, "message_approved"), message),
        (),
    );
}

#[test]
fn rotate_signers_bypass_rotation_delay() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator.clone(), 1, 5);
    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers.clone());
    let bypass_rotation_delay = true;
    let new_epoch: u64 = client.epoch() + 1;

    client
        .mock_auths(&[MockAuth {
            address: &operator,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "rotate_signers",
                args: (
                    new_signers.signers.clone(),
                    proof.clone(),
                    bypass_rotation_delay,
                )
                    .into_val(&env),
                sub_invokes: &[],
            },
        }])
        .rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "signers_rotated"),
            new_epoch,
            new_signers.signers.hash(&env),
        ),
        (),
    );
}

#[test]
fn rotate_signers_fail_not_latest_signers() {
    let (env, _contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator, 1, 5);
    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers.clone());
    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);

    let second_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = second_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers.clone());

    assert_contract_err!(
        client.try_rotate_signers(&second_signers.signers, &proof, &bypass_rotation_delay),
        ContractError::NotLatestSigners
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")] // Unauthorized
fn rotate_signers_bypass_rotation_delay_fail_if_not_operator() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let user = Address::generate(&env);
    let signers = initialize(&env, &client, owner, operator.clone(), 1, 5);
    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers);
    let bypass_rotation_delay = true;

    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "rotate_signers",
                args: (
                    new_signers.signers.clone(),
                    proof.clone(),
                    bypass_rotation_delay,
                )
                    .into_val(&env),
                sub_invokes: &[],
            },
        }])
        .rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);
}

#[test]
fn transfer_operatorship() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let new_operator = Address::generate(&env);

    initialize(&env, &client, owner, operator.clone(), 1, randint(1, 10));

    assert_eq!(client.operator(), operator);

    client
        .mock_auths(&[MockAuth {
            address: &operator,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_operatorship",
                args: (&new_operator,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_operatorship(&new_operator);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "operatorship_transferred"),
            operator.clone(),
            new_operator.clone(),
        ),
        (),
    );

    assert_eq!(client.operator(), new_operator);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")] // Unauthorized
fn transfer_operatorship_unauthorized() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let new_operator = Address::generate(&env);
    let user = Address::generate(&env);

    initialize(&env, &client, owner, operator.clone(), 1, randint(1, 10));

    assert_eq!(client.operator(), operator);
    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_operatorship",
                args: (&new_operator,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_operatorship(&new_operator);
}

#[test]
fn transfer_ownership() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let new_owner = Address::generate(&env);

    initialize(&env, &client, owner.clone(), operator, 1, randint(1, 10));

    assert_eq!(client.owner(), owner);

    client
        .mock_auths(&[MockAuth {
            address: &owner,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_ownership",
                args: (&new_owner,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_ownership(&new_owner);

    assert_last_emitted_event(
        &env,
        &contract_id,
        (
            Symbol::new(&env, "ownership_transferred"),
            owner,
            new_owner.clone(),
        ),
        (),
    );

    assert_eq!(client.owner(), new_owner);
}

#[test]
#[should_panic(expected = "HostError: Error(Auth, InvalidAction)")] // Unauthorized
fn transfer_ownership_unauthorized() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let new_owner = Address::generate(&env);
    let user = Address::generate(&env);

    initialize(&env, &client, owner.clone(), operator, 1, randint(1, 10));

    assert_eq!(client.owner(), owner);
    client
        .mock_auths(&[MockAuth {
            address: &user,
            invoke: &MockAuthInvoke {
                contract: &contract_id,
                fn_name: "transfer_ownership",
                args: (&new_owner,).into_val(&env),
                sub_invokes: &[],
            },
        }])
        .transfer_ownership(&new_owner);
}
