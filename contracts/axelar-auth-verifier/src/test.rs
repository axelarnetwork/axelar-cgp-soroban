#![cfg(test)]
extern crate std;

use rand::Rng;
use soroban_sdk::{symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events}, vec, xdr::{FromXdr, ToXdr}, Address, Bytes, Env, Vec};

use axelar_soroban_std::testutils::{assert_invocation, assert_emitted_event};

use crate::{contract::{AxelarAuthVerifier, AxelarAuthVerifierClient}, testutils::{generate_proof, generate_signer_set, TestSignerSet}, types::WeightedSigners};

fn setup_env<'a>() -> (Env, Address, AxelarAuthVerifierClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarAuthVerifier);
    let client = AxelarAuthVerifierClient::new(&env, &contract_id);

    (env, contract_id, client)
}

pub fn initialize(env: &Env, client: &AxelarAuthVerifierClient, owner: Address, previous_signer_retention: u32, num_signers: u32) -> TestSignerSet {
    let signers = generate_signer_set(env, num_signers);
    let signer_sets = vec![&env, signers.signer_set.clone()].to_xdr(env);
    let signer_set_hash = env.crypto().keccak256(&signers.signer_set.clone().to_xdr(env));

    client.initialize(&owner, &previous_signer_retention, &signer_sets);

    assert_emitted_event(env, env.events().all().len() - 1, &client.address,
        (symbol_short!("transfer"), signer_set_hash),
        (signers.signer_set.clone(),));

    signers
}

pub fn transfer_operatorship(env: &Env, client: &AxelarAuthVerifierClient, new_signers: TestSignerSet) -> Bytes {
    let encoded_new_signer_set = new_signers.signer_set.clone().to_xdr(env);

    client.transfer_operatorship(&encoded_new_signer_set);

    assert_emitted_event(env, env.events().all().len() - 1, &client.address,
        (symbol_short!("transfer"), env.crypto().keccak256(&encoded_new_signer_set)),
        (new_signers.signer_set.clone(),));

    encoded_new_signer_set
}

fn randint(a: u32, b: u32) -> u32 {
    rand::thread_rng().gen_range(a..b)
}

#[test]
fn test_initialize() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    initialize(&env, &client, user, randint(0, 10), randint(1, 10));
}

#[test]
fn test_validate_proof() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let msg = Bytes::from_array(&env, &[0x01, 0x02, 0x03]);
    let msg_hash = env.crypto().keccak256(&msg);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    // validate_proof shouldn't panic
    let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(latest_signer_set);
}

#[test]
fn test_transfer_operatorship() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(&env, &client, user.clone(), previous_signer_retention, randint(1, 10));

    let msg = Bytes::from_array(&env, &[0x01, 0x02, 0x03]);
    let msg_hash = env.crypto().keccak256(&msg);

    let new_signers = generate_signer_set(&env, randint(1, 10));

    let encoded_new_signer_set = transfer_operatorship(&env, &client, new_signers.clone());

    assert_invocation(&env, &user, &client.address, "transfer_operatorship", (encoded_new_signer_set,));

    let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());
    let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(latest_signer_set);
}

#[test]
fn test_multi_transfer_operatorship() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = randint(0, 5);

    let original_signers = initialize(&env, &client, user, previous_signer_retention, randint(1, 10));

    let msg = Bytes::from_array(&env, &[0x01, 0x02, 0x03]);
    let msg_hash = env.crypto().keccak256(&msg);

    let mut previous_signers = original_signers.clone();

    for _ in 0..previous_signer_retention {
        let new_signers = generate_signer_set(&env, randint(1, 10));

        transfer_operatorship(&env, &client, new_signers.clone());

        let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());
        let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
        assert!(latest_signer_set);

        let proof = generate_proof(&env, msg_hash.clone(), previous_signers.clone());
        let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
        assert!(!latest_signer_set);

        previous_signers = new_signers;
    }

    // Proof from the first signer set should still be valid
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());
    let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(!latest_signer_set);
}

#[test]
fn test_transfer_operatorship_panics_on_outdated_signer_set() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = randint(0, 5);

    let original_signers = initialize(&env, &client, user, previous_signer_retention, randint(1, 10));

    let msg = Bytes::from_array(&env, &[0x01, 0x02, 0x03]);
    let msg_hash = env.crypto().keccak256(&msg);

    for _ in 0..(previous_signer_retention + 1) {
        let new_signers = generate_signer_set(&env, randint(1, 10));
        transfer_operatorship(&env, &client, new_signers.clone());
    }

    // Proof from the first signer set should fail
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());
    let res = client.try_validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(res.is_err());
}
