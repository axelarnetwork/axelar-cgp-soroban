use crate::error::ContractError;
use crate::types::{ProofSignature, ProofSigner, WeightedSigner, WeightedSigners};

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, Vec,
};

use axelar_soroban_std::{assert_err, assert_ok};

use crate::{
    auth::{self, initialize_auth},
    contract::{AxelarGateway, AxelarGatewayClient},
    testutils::{self, generate_proof, generate_signers_set, initialize, randint},
};

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    (env, contract_id, client)
}

#[test]
fn test_initialize() {
    let (env, _, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );
}

#[test]
fn fails_with_empty_signer_set() {
    let (env, contract_id, _client) = setup_env();

    // create an empty WeightedSigners vector
    let empty_signer_set = Vec::<WeightedSigners>::new(&env);
    let domain_separator: BytesN<32> = BytesN::random(&env);
    let previous_signer_retention = randint(0, 10) as u64;
    let minimum_rotation_delay = 0;
    let initial_signers = empty_signer_set;

    // call should panic because signer set is empty
    env.as_contract(&contract_id, || {
        assert_err!(
            initialize_auth(
                env.clone(),
                domain_separator,
                minimum_rotation_delay,
                previous_signer_retention,
                initial_signers,
            ),
            ContractError::InvalidSigners
        );
    })
}

#[test]
fn validate_proof() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    // validate_proof shouldn't panic
    env.as_contract(&contract_id, || {
        assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
    });
}

#[test]
fn fail_validate_proof_invalid_epoch() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let different_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash.clone(), different_signers);

    // should panic, epoch should return zero for unknown signer set
    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSigners
        );
    })
}

#[test]
#[should_panic(expected = "failed ED25519 verification")]
fn fail_validate_proof_invalid_signatures() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    let different_msg_hash: BytesN<32> = BytesN::random(&env);

    // should panic, proof is for different message hash
    // NOTE: panic occurs in std function cannot handle explicitly
    env.as_contract(&contract_id, || {
        assert_ok!(auth::validate_proof(&env, &different_msg_hash, proof));
    })
}

#[test]
fn fail_validate_proof_empty_signatures() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    // Modify signatures to make them invalid
    let mut new_signers = Vec::new(&env);
    for signer in proof.signers.iter() {
        new_signers.push_back(ProofSigner {
            signer: signer.signer,
            signature: ProofSignature::Unsigned,
        });
    }
    proof.signers = new_signers;

    // validate_proof should panic, empty signatures
    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSignatures
        );
    })
}

#[test]
fn fail_validate_proof_invalid_signer_set() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );
    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator.clone());

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    let new_proof = generate_proof(&env, msg_hash.clone(), new_signers);

    proof.signers = new_proof.signers;

    // validate_proof should panic, signatures do not match signers
    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSigners
        );
    })
}

#[test]
fn fail_validate_proof_threshold_not_met() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let mut total_weight = 0u128;

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    // Modify signatures to make them invalid
    let mut new_signers = Vec::new(&env);
    for ProofSigner { signer, signature } in proof.signers.iter() {
        total_weight += signer.weight;

        if total_weight < proof.threshold {
            new_signers.push_back(ProofSigner { signer, signature });
        } else {
            new_signers.push_back(ProofSigner {
                signer,
                signature: ProofSignature::Unsigned,
            });
        }
    }
    proof.signers = new_signers;

    // should panic, all signatures are valid but total weight is below threshold
    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSignatures
        );
    })
}
#[test]
fn fail_validate_proof_threshold_overflow() {
    let (env, contract_id, client) = setup_env();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let mut signers = initialize(
        &env,
        &client,
        owner,
        operator,
        randint(0, 10),
        randint(1, 10),
    );

    let last_index = signers.signers.signers.len() - 1;

    // get last signer and modify its weight to max u128 - 1
    if let Some(mut last_signer) = signers.signers.signers.get(last_index) {
        last_signer.weight = u128::MAX - 1;
        signers.signers.signers.set(last_index, last_signer);
    }

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash.clone(), signers);

    // should panic, as modified signer wouldn't match the epoch
    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSigners
        );
    });
}

#[test]
fn test_rotate_signers() {
    let (env, contract_id, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

    testutils::rotate_signers(&env, &contract_id, new_signers.clone());

    let proof = generate_proof(&env, msg_hash.clone(), new_signers);

    env.as_contract(&contract_id, || {
        assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
    });
}

#[test]
fn rotate_signers_fail_empty_signers() {
    let (env, _, _client) = setup_env();

    let empty_signers = WeightedSigners {
        signers: Vec::<WeightedSigner>::new(&env),
        threshold: 0u128,
        nonce: BytesN::random(&env),
    };

    // should throw an error, empty signer set
    assert_err!(
        auth::rotate_signers(&env, &empty_signers, false),
        ContractError::InvalidSigners
    );
}

#[test]
fn rotate_signers_fail_zero_weight() {
    let (env, _, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let last_index = new_signers.signers.signers.len() - 1;

    // get last signer and modify its weight to zero
    if let Some(mut last_signer) = new_signers.signers.signers.get(last_index) {
        last_signer.weight = 0u128;
        new_signers.signers.signers.set(last_index, last_signer);
    }

    // should throw an error, last signer weight is zero
    assert_err!(
        auth::rotate_signers(&env, &new_signers.signers, false),
        ContractError::InvalidWeight
    )
}

#[test]
fn rotate_signers_fail_weight_overflow() {
    let (env, _, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signers_set(&env, randint(3, 10), BytesN::random(&env));

    let last_index = new_signers.signers.signers.len() - 1;

    // get last signer and modify its weight to max u128 - 1
    if let Some(mut last_signer) = new_signers.signers.signers.get(last_index) {
        last_signer.weight = u128::MAX - 1;
        new_signers.signers.signers.set(last_index, last_signer);
    }

    // last signer weight should cause overflow
    assert_err!(
        auth::rotate_signers(&env, &new_signers.signers, false),
        ContractError::WeightOverflow
    )
}

#[test]
fn rotate_signers_fail_zero_threshold() {
    let (env, _, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    // set the threshold to zero
    new_signers.signers.threshold = 0u128;

    // should error because the threshold is set to zero
    assert_err!(
        auth::rotate_signers(&env, &new_signers.signers, false),
        ContractError::InvalidThreshold
    );
}

#[test]
fn rotate_signers_fail_low_total_weight() {
    let (env, _, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let total_weight = new_signers
        .signers
        .signers
        .iter()
        .map(|WeightedSigner { weight, .. }| weight)
        .reduce(|acc, weight| acc + weight)
        .expect("Empty signers");

    let new_threshold = total_weight + 1;

    // set the threshold to zero
    new_signers.signers.threshold = new_threshold;

    // should error because the threshold is set to zero
    assert_err!(
        auth::rotate_signers(&env, &new_signers.signers, false),
        ContractError::InvalidThreshold
    )
}

#[test]
fn rotate_signers_fail_wrong_signer_order() {
    let (env, _, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let min_signers = 2; // need at least 2 signers to test incorrect ordering
    let mut new_signers =
        generate_signers_set(&env, randint(min_signers, 10), BytesN::random(&env));

    let len = new_signers.signers.signers.len();

    // create a new vec and reverse signer order
    let mut reversed_signers = Vec::new(&env);
    for i in (0..len).rev() {
        if let Some(item) = new_signers.signers.signers.get(i) {
            reversed_signers.push_back(item);
        }
    }

    new_signers.signers.signers = reversed_signers;

    // should error because signers are in wrong order
    assert_err!(
        auth::rotate_signers(&env, &new_signers.signers, false),
        ContractError::InvalidSigners
    )
}

#[test]
fn multi_rotate_signers() {
    let (env, contract_id, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = randint(1, 5);

    let original_signers = initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);

    let mut previous_signers = original_signers.clone();

    for _ in 0..previous_signer_retention {
        let new_signers = generate_signers_set(
            &env,
            randint(1, 10),
            original_signers.domain_separator.clone(),
        );

        testutils::rotate_signers(&env, &contract_id, new_signers.clone());

        let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());

        env.as_contract(&contract_id, || {
            assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        });

        let proof = generate_proof(&env, msg_hash.clone(), previous_signers.clone());

        env.as_contract(&contract_id, || {
            assert!(!assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        });

        previous_signers = new_signers;
    }

    // Proof from the first signer set should still be valid
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());
    env.as_contract(&contract_id, || {
        assert!(!assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
    })
}

#[test]
fn rotate_signers_panics_on_outdated_signer_set() {
    let (env, contract_id, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = randint(0, 5);

    let original_signers = initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash: BytesN<32> = BytesN::random(&env);

    for _ in 0..(previous_signer_retention + 1) {
        let new_signers = generate_signers_set(
            &env,
            randint(1, 10),
            original_signers.domain_separator.clone(),
        );
        testutils::rotate_signers(&env, &contract_id, new_signers.clone());
    }

    // Proof from the first signer set should fail
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());

    env.as_contract(&contract_id, || {
        assert_err!(
            auth::validate_proof(&env, &msg_hash, proof),
            ContractError::InvalidSigners
        )
    });
}

#[test]
fn rotate_signers_fail_duplicated_signers() {
    let (env, contract_id, client) = setup_env();

    let owner = Address::generate(&env);
    let operator = Address::generate(&env);
    let previous_signer_retention = 1;

    let signers = initialize(
        &env,
        &client,
        owner,
        operator,
        previous_signer_retention,
        randint(1, 10),
    );

    let msg_hash = BytesN::random(&env);
    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);
    let duplicated_signers = new_signers.clone();

    testutils::rotate_signers(&env, &contract_id, new_signers.clone());

    let proof = generate_proof(&env, msg_hash.clone(), new_signers);

    env.as_contract(&contract_id, || {
        assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
    });

    // should panic, duplicated signers

    env.as_contract(&contract_id, || {
        assert_err!(
            auth::rotate_signers(&env, &duplicated_signers.signers, false),
            ContractError::DuplicateSigners
        );
    });
}
