#![cfg(test)]
extern crate std;

use soroban_sdk::{
    symbol_short, testutils::Address as _, xdr::ToXdr, Address, Bytes, Env, Vec, U256,
};

use axelar_soroban_std::{assert_emitted_event, testutils::assert_invocation};

use crate::{
    contract::{AxelarAuthVerifier, AxelarAuthVerifierClient},
    testutils::{
        generate_empty_signer_set, generate_proof, generate_random_payload_and_hash,
        generate_signer_set, initialize, randint, transfer_operatorship,
    },
};

fn setup_env<'a>() -> (Env, Address, AxelarAuthVerifierClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarAuthVerifier);
    let client = AxelarAuthVerifierClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
fn test_initialize() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    initialize(&env, &client, user, randint(0, 10), randint(1, 10));
}

#[test]
#[should_panic(expected = "Already initialized")]
fn fails_if_already_initialized() {
    let (env, _, client) = setup_env();
    let user_one = Address::generate(&env);
    let user_two = Address::generate(&env);

    initialize(&env, &client, user_one, randint(0, 10), randint(1, 10));

    // second initialization should panic
    initialize(&env, &client, user_two, randint(0, 10), randint(1, 10));
}

#[test]
fn fails_with_empty_signer_set() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);

    // create an empty WeightedSigners vector
    let empty_signer_set = generate_empty_signer_set(&env);

    // serialize the empty signer set to Bytes
    let empty_operator_set = empty_signer_set.to_xdr(&env);

    // call should panic because signer set is empty
    let res = client.try_initialize(&owner, &randint(0, 10), &empty_operator_set);
    assert!(res.is_err());
}

#[test]
fn transfer_ownership() {
    let (env, _, client) = setup_env();

    let initial_owner = Address::generate(&env);
    let new_owner = Address::generate(&env);

    initialize(
        &env,
        &client,
        initial_owner.clone(),
        randint(1, 10),
        randint(1, 10),
    );

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
        -1,
        &client.address,
        (symbol_short!("ownership"), initial_owner, new_owner.clone()),
        (),
    );

    let retrieved_owner = client.owner();
    assert_eq!(retrieved_owner, new_owner);
}

#[test]
fn validate_proof() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let msg_hash = generate_random_payload_and_hash(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    // validate_proof shouldn't panic
    let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(latest_signer_set);
}

#[test]
#[should_panic(expected = "unknown signer set")]
fn fail_validate_proof_invalid_epoch() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let different_signers = generate_signer_set(&env, randint(1, 10));

    let msg_hash = generate_random_payload_and_hash(&env);
    let proof = generate_proof(&env, msg_hash.clone(), different_signers);

    // should panic, epoch should return zero for unknown signer set
    client.validate_proof(&msg_hash, &proof.to_xdr(&env));
}

#[test]
#[should_panic(expected = "invalid signatures")]
fn fail_validate_proof_invalid_signatures() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let msg_hash = generate_random_payload_and_hash(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    let different_msg = Bytes::from_array(&env, &[0x04, 0x05, 0x06]);
    let different_msg_hash = env.crypto().keccak256(&different_msg);

    // should panic, proof is for different message hash
    client.validate_proof(&different_msg_hash, &proof.to_xdr(&env));
}

#[test]
#[should_panic(expected = "invalid signatures")]
fn fail_validate_proof_empty_signatures() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let msg_hash = generate_random_payload_and_hash(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    proof.signatures = Vec::new(&env);

    // validate_proof should panic, empty signatures
    client.validate_proof(&msg_hash, &proof.to_xdr(&env));
}

#[test]
#[should_panic(expected = "invalid signatures")]
fn fail_validate_proof_invalid_signer_set() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let msg_hash = generate_random_payload_and_hash(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    let new_signers = generate_signer_set(&env, randint(1, 10));
    let new_proof = generate_proof(&env, msg_hash.clone(), new_signers);

    proof.signatures = new_proof.signatures;

    // validate_proof should panic, signatures do not match signers
    client.validate_proof(&msg_hash, &proof.to_xdr(&env));
}

#[test]
#[should_panic(expected = "invalid signatures")]
fn fail_validate_proof_threshold_not_met() {
    let (env, _, client) = setup_env();
    let user = Address::generate(&env);

    let signers = initialize(&env, &client, user, randint(0, 10), randint(1, 10));

    let env = &env;
    let zero = U256::from_u32(env, 0);
    let mut total_weight = zero.clone();

    let mut index_below_threshold = 0;

    // find the index where the total weight is just below the threshold
    for (i, weight) in signers.signer_set.signers.iter().map(|s| s.1).enumerate() {
        total_weight = total_weight.add(&weight);
        if total_weight >= signers.signer_set.threshold {
            index_below_threshold = i;
            break;
        }
    }

    let msg_hash = generate_random_payload_and_hash(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    // remove signatures to just below the threshold
    proof.signatures = proof.signatures.slice(0..index_below_threshold as u32);

    // should panic, all signatures are valid but total weight is below threshold
    client.validate_proof(&msg_hash, &proof.to_xdr(&env));
}

#[test]
fn test_transfer_operatorship() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash = generate_random_payload_and_hash(&env);

    let new_signers = generate_signer_set(&env, randint(1, 10));

    let encoded_new_signer_set = transfer_operatorship(&env, &client, new_signers.clone());

    assert_invocation(
        &env,
        &user,
        &client.address,
        "transfer_operatorship",
        (encoded_new_signer_set,),
    );

    let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());
    let latest_signer_set = client.validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(latest_signer_set);
}

#[test]
fn transfer_operatorship_fail_empty_signer_set() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let empty_signer_set = generate_empty_signer_set(&env);

    let empty_operator_set = empty_signer_set.to_xdr(&env);

    // should throw an error, empty signer set
    let res = client.try_transfer_operatorship(&empty_operator_set);
    assert!(res.is_err());
}

#[test]
fn transfer_operatorship_fail_zero_weight() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signer_set(&env, randint(1, 10));

    let last_index = new_signers.signer_set.signers.len() as u32 - 1;

    // get last signer and modify its weight to zero
    if let Some(mut last_signer) = new_signers.signer_set.signers.get(last_index) {
        last_signer.1 = U256::from_u32(&env, 0);
        new_signers.signer_set.signers.set(last_index, last_signer);
    }

    let encoded_new_signer_set = new_signers.signer_set.to_xdr(&env);

    // should throw an error, last signer weight is zero
    let res = client.try_transfer_operatorship(&encoded_new_signer_set);
    assert!(res.is_err());
}

#[test]
fn transfer_operatorship_fail_zero_threshold() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signer_set(&env, randint(1, 10));

    // set the threshold to zero
    new_signers.signer_set.threshold = U256::from_u32(&env, 0);

    let encoded_new_signer_set = new_signers.signer_set.to_xdr(&env);

    // should error because the threshold is set to zero
    let res = client.try_transfer_operatorship(&encoded_new_signer_set);
    assert!(res.is_err());
}

#[test]
fn transfer_operatorship_fail_low_total_weight() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signer_set(&env, randint(1, 10));

    let one = U256::from_u32(&env, 1);

    let total_weight = new_signers
        .signer_set
        .signers
        .iter()
        .map(|(_, weight)| weight)
        .reduce(|acc, weight| acc.add(&weight))
        .expect("Empty signers");

    let new_threshold = total_weight.add(&one);

    // set the threshold to zero
    new_signers.signer_set.threshold = new_threshold;

    let encoded_new_signer_set = new_signers.signer_set.to_xdr(&env);

    // should error because the threshold is set to zero
    let res = client.try_transfer_operatorship(&encoded_new_signer_set);
    assert!(res.is_err());
}

#[test]
fn transfer_operatorship_fail_wrong_signer_order() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        user.clone(),
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signer_set(&env, randint(1, 10));

    let len = new_signers.signer_set.signers.len();

    // create a new vec and reverse signer order
    let mut reversed_signers = Vec::new(&env);
    for i in (0..len).rev() {
        if let Some(item) = new_signers.signer_set.signers.get(i as u32) {
            reversed_signers.push_back(item);
        }
    }

    new_signers.signer_set.signers = reversed_signers;

    let encoded_new_signer_set = new_signers.signer_set.to_xdr(&env);

    // should error because signers are in wrong order
    let res = client.try_transfer_operatorship(&encoded_new_signer_set);
    assert!(res.is_err());
}

#[test]
fn multi_transfer_operatorship() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = randint(1, 5);

    let original_signers = initialize(
        &env,
        &client,
        user,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash = generate_random_payload_and_hash(&env);

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
fn transfer_operatorship_panics_on_outdated_signer_set() {
    let (env, _, client) = setup_env();

    let user = Address::generate(&env);
    let previous_signer_retention = randint(0, 5);

    let original_signers = initialize(
        &env,
        &client,
        user,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash = generate_random_payload_and_hash(&env);

    for _ in 0..(previous_signer_retention + 1) {
        let new_signers = generate_signer_set(&env, randint(1, 10));
        transfer_operatorship(&env, &client, new_signers.clone());
    }

    // Proof from the first signer set should fail
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());
    let res = client.try_validate_proof(&msg_hash, &proof.to_xdr(&env));
    assert!(res.is_err());
}
