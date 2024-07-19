#![cfg(test)]
extern crate std;

use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contractimpl, symbol_short, testutils::Address as _, Address, Env, String, Vec,
};

use axelar_soroban_std::assert_emitted_event;
use axelar_soroban_std::types::Hash;

use crate::contract::{AxelarServiceGovernance, AxelarServiceGovernanceClient};
use crate::types::{GovernanceProposal, ServiceGovernanceCommandType};

// Mock target contract
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

// Mock Axelar Gateway
#[contract]
pub struct MockAxelarGateway;

#[contractimpl]
impl MockAxelarGateway {
    pub fn validate_message(
        _env: Env,
        _caller: Address,
        _message_id: String,
        _source_chain: String,
        _source_address: String,
        _payload_hash: Hash,
    ) -> bool {
        true
    }
}

fn setup_env<'a>() -> (
    Env,
    Address,
    AxelarServiceGovernanceClient<'a>,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarServiceGovernance);
    let client = AxelarServiceGovernanceClient::new(&env, &contract_id);

    let gateway_id = env.register_contract(None, MockAxelarGateway);
    let target_contract_id = env.register_contract(None, TestTarget);

    (env, contract_id, client, gateway_id, target_contract_id)
}

#[test]
fn test_initialize() {
    let (env, _, client, gateway, _) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);
}
#[test]
#[should_panic(expected = "Already initialized")]
fn fail_already_initialized() {
    let (env, _, client, gateway, _) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    client.initialize(&multisig, &gateway, &minimum_time_delay);
}

#[test]
fn test_execute_proposal_time_lock() {
    let (env, contract_id, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        command: ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64,
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: env.ledger().timestamp() + minimum_time_delay,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    let proposal_eta = client.get_proposal_eta(&target, &proposal.func, &proposal.args);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("timelock"), symbol_short!("added")),
        (target, proposal.func, proposal.args, proposal.eta),
    );

    assert_eq!(proposal_eta, proposal.eta);
}

#[test]
fn test_execute_proposal_approve_multisig() {
    let (env, contract_id, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        command: ServiceGovernanceCommandType::ApproveMultisigProposal as u64,
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: 0,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    let is_approved: bool =
        client.is_multisig_proposal_approved(&target, &proposal.func, &proposal.args);

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("multisig"), symbol_short!("added")),
        (target, proposal.func, proposal.args),
    );

    assert!(is_approved);
}

#[test]
fn test_execute_time_locked_proposal() {
    let (env, contract_id, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 0_u64; // No delay for testing

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: env.ledger().timestamp() + minimum_time_delay,
        command: ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    client.execute_proposal(&target, &proposal.func, &proposal.args);

    assert_emitted_event(&env, -2, &target, (symbol_short!("executed"),), ());

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("timelock"), symbol_short!("executed")),
        (target.clone(), proposal.func, proposal.args),
    );
}

#[test]
fn fail_execute_time_locked_proposal_not_ready() {
    let (env, _, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: env.ledger().timestamp() + minimum_time_delay,
        command: ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    let res = client.try_execute_proposal(&target, &proposal.func, &proposal.args);

    assert!(res.is_err());
}

#[test]
fn test_execute_multisig_proposal() {
    let (env, contract_id, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: 0,
        command: ServiceGovernanceCommandType::ApproveMultisigProposal as u64,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    client.execute_multisig_proposal(&target, &proposal.func, &proposal.args);

    assert_emitted_event(&env, -2, &target, (symbol_short!("executed"),), ());

    assert_emitted_event(
        &env,
        -1,
        &contract_id,
        (symbol_short!("multisig"), symbol_short!("executed")),
        (target.clone(), proposal.func, proposal.args),
    );
}

#[test]
fn fail_execute_proposal_not_found() {
    let (env, _, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 0_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        command: ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64,
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: env.ledger().timestamp() + minimum_time_delay,
    };

    let res = client.try_execute_proposal(&target, &proposal.func, &proposal.args);

    assert!(res.is_err());
}

#[test]
fn fail_execute_proposal_replay() {
    let (env, _, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 0_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        command: ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64,
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: env.ledger().timestamp() + minimum_time_delay,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    client.execute_proposal(&target, &proposal.func, &proposal.args);

    let res = client.try_execute_proposal(&target, &proposal.func, &proposal.args);

    assert!(res.is_err());
}

#[test]
fn fail_execute_multisig_proposal_not_found() {
    let (env, _, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: 0,
        command: ServiceGovernanceCommandType::ApproveMultisigProposal as u64,
    };

    let res = client.try_execute_multisig_proposal(&target, &proposal.func, &proposal.args);

    assert!(res.is_err());
}

#[test]
fn fail_execute_multisig_proposal_replay() {
    let (env, _, client, gateway, target) = setup_env();
    let multisig = Address::generate(&env);
    let minimum_time_delay = 10000_u64;

    client.initialize(&multisig, &gateway, &minimum_time_delay);

    let proposal = GovernanceProposal {
        target: target.clone(),
        func: symbol_short!("method"),
        args: Vec::new(&env),
        eta: 0,
        command: ServiceGovernanceCommandType::ApproveMultisigProposal as u64,
    };

    let proposal_payload = proposal.clone().to_xdr(&env);
    let message_id = String::from_str(&env, "message1");
    let source_chain = String::from_str(&env, "chain1");
    let source_address = String::from_str(&env, "address1");

    client.execute(
        &message_id,
        &source_chain,
        &source_address,
        &proposal_payload,
    );

    client.execute_multisig_proposal(&target, &proposal.func, &proposal.args);

    let res = client.try_execute_multisig_proposal(&target, &proposal.func, &proposal.args);

    assert!(res.is_err());
}
