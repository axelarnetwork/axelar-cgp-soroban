use std::format;

use stellar_axelar_std::testutils::Address as _;
use stellar_axelar_std::{
    assert_auth, assert_auth_err, assert_contract_err, events, Address, BytesN,
};

use super::utils::setup_env;
use crate::error::ContractError;
use crate::event::{FlowLimiterAddedEvent, FlowLimiterRemovedEvent};

const fn dummy_flow_limit() -> i128 {
    1000
}

#[test]
fn add_flow_limiter_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    assert!(!client.is_flow_limiter(&token_id, &flow_limiter));

    assert_auth!(
        client.operator(),
        client.add_flow_limiter(&token_id, &flow_limiter)
    );

    goldie::assert!(events::fmt_last_emitted_event::<FlowLimiterAddedEvent>(
        &env
    ));

    assert!(client.is_flow_limiter(&token_id, &flow_limiter));
}

#[test]
fn add_flow_limiter_fails_if_not_operator() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    let not_operator = Address::generate(&env);
    assert_auth_err!(
        not_operator,
        client.add_flow_limiter(&token_id, &flow_limiter)
    );
    assert_auth_err!(
        client.owner(),
        client.add_flow_limiter(&token_id, &flow_limiter)
    );
}

#[test]
fn add_flow_limiter_fails_if_already_set() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_add_flow_limiter(&token_id, &flow_limiter),
        ContractError::FlowLimiterAlreadySet
    );
}

#[test]
fn remove_flow_limiter_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);
    assert!(client.is_flow_limiter(&token_id, &flow_limiter));

    assert_auth!(
        client.operator(),
        client.remove_flow_limiter(&token_id, &flow_limiter)
    );

    goldie::assert!(events::fmt_last_emitted_event::<FlowLimiterRemovedEvent>(
        &env
    ));

    assert!(!client.is_flow_limiter(&token_id, &flow_limiter));
}

#[test]
fn remove_flow_limiter_fails_if_not_operator() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);

    let not_operator = Address::generate(&env);
    assert_auth_err!(
        not_operator,
        client.remove_flow_limiter(&token_id, &flow_limiter)
    );
    assert_auth_err!(
        client.owner(),
        client.remove_flow_limiter(&token_id, &flow_limiter)
    );
}

#[test]
fn remove_flow_limiter_fails_if_not_set() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_remove_flow_limiter(&token_id, &flow_limiter),
        ContractError::FlowLimiterNotSet
    );
}

#[test]
fn transfer_flow_limiter_succeeds() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mock_all_auths().add_flow_limiter(&token_id, &from);

    assert_auth!(
        client.operator(),
        client.transfer_flow_limiter(&token_id, &from, &to)
    );

    let removed = events::fmt_emitted_event_at_idx::<FlowLimiterRemovedEvent>(&env, -2);
    let added = events::fmt_emitted_event_at_idx::<FlowLimiterAddedEvent>(&env, -1);
    goldie::assert!(format!("{}\n---\n{}", removed, added));

    assert!(!client.is_flow_limiter(&token_id, &from));
    assert!(client.is_flow_limiter(&token_id, &to));
}

#[test]
fn transfer_flow_limiter_fails_if_from_equals_to() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_flow_limiter(&token_id, &flow_limiter, &flow_limiter),
        ContractError::FlowLimiterAlreadySet
    );

    assert!(client.is_flow_limiter(&token_id, &flow_limiter));
}

#[test]
fn transfer_flow_limiter_fails_if_not_operator() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mock_all_auths().add_flow_limiter(&token_id, &from);

    let not_operator = Address::generate(&env);
    assert_auth_err!(
        not_operator,
        client.transfer_flow_limiter(&token_id, &from, &to)
    );
}

#[test]
fn transfer_flow_limiter_fails_if_from_not_set() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_flow_limiter(&token_id, &from, &to),
        ContractError::FlowLimiterNotSet
    );
}

#[test]
fn transfer_flow_limiter_fails_if_to_already_set() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let from = Address::generate(&env);
    let to = Address::generate(&env);

    client.mock_all_auths().add_flow_limiter(&token_id, &from);
    client.mock_all_auths().add_flow_limiter(&token_id, &to);

    assert_contract_err!(
        client
            .mock_all_auths()
            .try_transfer_flow_limiter(&token_id, &from, &to),
        ContractError::FlowLimiterAlreadySet
    );
}

#[test]
fn approved_flow_limiter_can_set_flow_limit() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);

    assert_auth!(
        &flow_limiter,
        client.set_flow_limit(&flow_limiter, &token_id, &Some(dummy_flow_limit()))
    );

    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit()));
}

#[test]
fn removed_flow_limiter_cannot_set_flow_limit() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &flow_limiter);
    client
        .mock_all_auths()
        .remove_flow_limiter(&token_id, &flow_limiter);

    assert_contract_err!(
        client.mock_all_auths().try_set_flow_limit(
            &flow_limiter,
            &token_id,
            &Some(dummy_flow_limit())
        ),
        ContractError::NotApprovedFlowLimiter
    );
}

#[test]
fn multiple_flow_limiters_per_token() {
    let (env, client, _, _, _) = setup_env();
    let token_id = BytesN::from_array(&env, &[1; 32]);
    let limiter_a = Address::generate(&env);
    let limiter_b = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &limiter_a);
    client
        .mock_all_auths()
        .add_flow_limiter(&token_id, &limiter_b);

    assert!(client.is_flow_limiter(&token_id, &limiter_a));
    assert!(client.is_flow_limiter(&token_id, &limiter_b));

    assert_auth!(
        &limiter_a,
        client.set_flow_limit(&limiter_a, &token_id, &Some(dummy_flow_limit()))
    );
    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit()));

    assert_auth!(
        &limiter_b,
        client.set_flow_limit(&limiter_b, &token_id, &Some(dummy_flow_limit() * 2))
    );
    assert_eq!(client.flow_limit(&token_id), Some(dummy_flow_limit() * 2));

    // Removing one does not affect the other
    client
        .mock_all_auths()
        .remove_flow_limiter(&token_id, &limiter_a);

    assert!(!client.is_flow_limiter(&token_id, &limiter_a));
    assert!(client.is_flow_limiter(&token_id, &limiter_b));

    assert_contract_err!(
        client.mock_all_auths().try_set_flow_limit(
            &limiter_a,
            &token_id,
            &Some(dummy_flow_limit())
        ),
        ContractError::NotApprovedFlowLimiter
    );
}

#[test]
fn flow_limiter_role_is_scoped_to_token_id() {
    let (env, client, _, _, _) = setup_env();
    let token_id_a = BytesN::from_array(&env, &[1; 32]);
    let token_id_b = BytesN::from_array(&env, &[2; 32]);
    let flow_limiter = Address::generate(&env);

    client
        .mock_all_auths()
        .add_flow_limiter(&token_id_a, &flow_limiter);

    assert!(client.is_flow_limiter(&token_id_a, &flow_limiter));
    assert!(!client.is_flow_limiter(&token_id_b, &flow_limiter));

    assert_contract_err!(
        client.mock_all_auths().try_set_flow_limit(
            &flow_limiter,
            &token_id_b,
            &Some(dummy_flow_limit())
        ),
        ContractError::NotApprovedFlowLimiter
    );
}
