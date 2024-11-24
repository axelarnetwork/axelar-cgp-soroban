use crate::error::ContractError;
use crate::types::{ProofSignature, ProofSigner, WeightedSigner};
use axelar_soroban_std::ensure;
use soroban_sdk::{crypto::Hash, Bytes, BytesN, Env, Vec};

use crate::event;
use crate::storage_types::DataKey;
use crate::types::{Proof, WeightedSigners};

pub fn initialize_auth(
    env: Env,
    domain_separator: BytesN<32>,
    minimum_rotation_delay: u64,
    previous_signer_retention: u64,
    initial_signers: Vec<WeightedSigners>,
) -> Result<(), ContractError> {
    env.storage().instance().set(&DataKey::Epoch, &0_u64);

    // TODO: Do we need to manually expose these in a query, or can it be read directly off of storage in Stellar?
    env.storage().instance().set(
        &DataKey::PreviousSignerRetention,
        &previous_signer_retention,
    );

    env.storage()
        .instance()
        .set(&DataKey::DomainSeparator, &domain_separator);

    env.storage()
        .instance()
        .set(&DataKey::MinimumRotationDelay, &minimum_rotation_delay);

    ensure!(!initial_signers.is_empty(), ContractError::InvalidSigners);

    for signers in initial_signers.into_iter() {
        rotate_signers(&env, &signers, false)?;
    }

    Ok(())
}

pub fn validate_proof(
    env: &Env,
    data_hash: &BytesN<32>,
    proof: Proof,
) -> Result<bool, ContractError> {
    let signers_set = proof.weighted_signers();

    let signers_hash = signers_set.hash(env);

    let signers_epoch = epoch_by_signers_hash(env, signers_hash.clone())?;

    let current_epoch = epoch(env);

    let is_latest_signers: bool = signers_epoch == current_epoch;

    let previous_signers_retention: u64 = env
        .storage()
        .instance()
        .get(&DataKey::PreviousSignerRetention)
        .expect("previous_signers_retention not found");

    ensure!(
        current_epoch - signers_epoch <= previous_signers_retention,
        ContractError::InvalidSigners
    );

    let msg_hash = message_hash_to_sign(env, signers_hash, data_hash);

    ensure!(
        validate_signatures(env, msg_hash, proof),
        ContractError::InvalidSignatures
    );

    Ok(is_latest_signers)
}

pub fn rotate_signers(
    env: &Env,
    new_signers: &WeightedSigners,
    enforce_rotation_delay: bool,
) -> Result<(), ContractError> {
    validate_signers(env, new_signers)?;

    update_rotation_timestamp(env, enforce_rotation_delay)?;

    let new_signers_hash = new_signers.hash(env);

    let new_epoch: u64 = epoch(env) + 1;

    env.storage().instance().set(&DataKey::Epoch, &new_epoch);

    env.storage()
        .persistent()
        .set(&DataKey::SignersHashByEpoch(new_epoch), &new_signers_hash);

    // signers must be distinct, since nonce should guarantee uniqueness even if signers are repeated
    ensure!(
        epoch_by_signers_hash(env, new_signers_hash.clone()).is_err(),
        ContractError::DuplicateSigners
    );

    env.storage().persistent().set(
        &DataKey::EpochBySignersHash(new_signers_hash.clone()),
        &new_epoch,
    );

    event::rotate_signers(env, new_epoch, new_signers_hash);

    Ok(())
}

pub fn epoch(env: &Env) -> u64 {
    env.storage()
        .instance()
        .get(&DataKey::Epoch)
        .expect("epoch not found")
}

pub fn epoch_by_signers_hash(env: &Env, signers_hash: BytesN<32>) -> Result<u64, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::EpochBySignersHash(signers_hash))
        .ok_or(ContractError::InvalidSignersHash)
}

pub fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError> {
    env.storage()
        .persistent()
        .get(&DataKey::SignersHashByEpoch(epoch))
        .ok_or(ContractError::InvalidEpoch)
}

fn message_hash_to_sign(env: &Env, signers_hash: BytesN<32>, data_hash: &BytesN<32>) -> Hash<32> {
    let domain_separator: BytesN<32> = env
        .storage()
        .instance()
        .get(&DataKey::DomainSeparator)
        .unwrap();

    let mut msg: Bytes = domain_separator.into();
    msg.extend_from_array(&signers_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());

    // TODO: use an appropriate non tx overlapping prefix
    env.crypto().keccak256(&msg)
}

fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) -> Result<(), ContractError> {
    let minimum_rotation_delay: u64 = env
        .storage()
        .instance()
        .get(&DataKey::MinimumRotationDelay)
        .expect("minimum_rotation_delay not found");

    let last_rotation_timestamp: u64 = env
        .storage()
        .instance()
        .get(&DataKey::LastRotationTimestamp)
        .unwrap_or(0);

    let current_timestamp = env.ledger().timestamp();

    if enforce_rotation_delay {
        ensure!(
            current_timestamp - last_rotation_timestamp >= minimum_rotation_delay,
            ContractError::InsufficientRotationDelay
        );
    }

    env.storage()
        .instance()
        .set(&DataKey::LastRotationTimestamp, &current_timestamp);

    Ok(())
}

fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
    let mut total_weight = 0u128;

    for ProofSigner {
        signer: WeightedSigner {
            signer: public_key,
            weight,
        },
        signature,
    } in proof.signers.iter()
    {
        if let ProofSignature::Signed(signature) = signature {
            env.crypto()
                .ed25519_verify(&public_key, msg_hash.to_bytes().as_ref(), &signature);

            total_weight = total_weight.checked_add(weight).unwrap();

            if total_weight >= proof.threshold {
                return true;
            }
        }
    }

    false
}

/// Check if signer set is valid, i.e signer/pub key hash are in sorted order,
/// weights are non-zero and sum to at least threshold
fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) -> Result<(), ContractError> {
    ensure!(
        !weighted_signers.signers.is_empty(),
        ContractError::InvalidSigners
    );

    // TODO: what's the min address/hash?
    let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
    let mut total_weight = 0u128;

    for signer in weighted_signers.signers.iter() {
        ensure!(
            previous_signer < signer.signer,
            ContractError::InvalidSigners
        );

        ensure!(signer.weight != 0, ContractError::InvalidWeight);

        previous_signer = signer.signer;
        total_weight = total_weight
            .checked_add(signer.weight)
            .ok_or(ContractError::WeightOverflow)?;
    }

    let threshold = weighted_signers.threshold;
    ensure!(
        threshold != 0 && total_weight >= threshold,
        ContractError::InvalidThreshold
    );

    Ok(())
}

#[cfg(all(test, feature = "testutils"))]
mod tests {
    use crate::error::ContractError;
    use crate::testutils::TestSignerSet;
    use crate::types::{ProofSignature, ProofSigner, WeightedSigner, WeightedSigners};
    use crate::AxelarGatewayClient;

    use soroban_sdk::{testutils::BytesN as _, BytesN, Env, Vec};

    use axelar_soroban_std::{assert_err, assert_ok};

    use crate::{
        auth::{self, initialize_auth},
        testutils::{self, generate_proof, generate_signers_set, randint, setup_gateway},
    };

    fn setup_env<'a>(
        previous_signers_retention: u32,
        num_signers: u32,
    ) -> (Env, TestSignerSet, AxelarGatewayClient<'a>) {
        let env = Env::default();
        env.mock_all_auths();
        let (signers, client) = setup_gateway(&env, previous_signers_retention, num_signers);

        (env, signers, client)
    }

    #[test]
    fn register_auth() {
        setup_env(randint(0, 10), randint(1, 10));
    }

    #[test]
    fn fails_with_empty_signer_set() {
        let (env, _signers, client) = setup_env(1, randint(1, 10));

        // create an empty WeightedSigners vector
        let empty_signer_set = Vec::<WeightedSigners>::new(&env);
        let domain_separator: BytesN<32> = BytesN::random(&env);
        let previous_signer_retention = randint(0, 10) as u64;
        let minimum_rotation_delay = 0;
        let initial_signers = empty_signer_set;

        // call should panic because signer set is empty
        env.as_contract(&client.address, || {
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
        let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let proof = generate_proof(&env, msg_hash.clone(), signers);

        // validate_proof shouldn't panic
        env.as_contract(&client.address, || {
            assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        });
    }

    #[test]
    fn fail_validate_proof_invalid_epoch() {
        let (env, _signers, client) = setup_env(randint(0, 10), randint(1, 10));

        let different_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let proof = generate_proof(&env, msg_hash.clone(), different_signers);

        // should panic, epoch should return zero for unknown signer set
        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSignersHash
            );
        })
    }

    #[test]
    #[should_panic(expected = "failed ED25519 verification")]
    fn fail_validate_proof_invalid_signatures() {
        let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let proof = generate_proof(&env, msg_hash.clone(), signers);

        let different_msg_hash: BytesN<32> = BytesN::random(&env);

        // should panic, proof is for different message hash
        // NOTE: panic occurs in std function cannot handle explicitly
        env.as_contract(&client.address, || {
            assert_ok!(auth::validate_proof(&env, &different_msg_hash, proof));
        })
    }

    #[test]
    fn fail_validate_proof_empty_signatures() {
        let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

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
        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSignatures
            );
        })
    }

    #[test]
    fn fail_validate_proof_invalid_signer_set() {
        let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

        let new_signers =
            generate_signers_set(&env, randint(1, 10), signers.domain_separator.clone());

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let mut proof = generate_proof(&env, msg_hash.clone(), signers);

        let new_proof = generate_proof(&env, msg_hash.clone(), new_signers);

        proof.signers = new_proof.signers;

        // validate_proof should panic, signatures do not match signers
        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSignersHash
            );
        })
    }

    #[test]
    fn fail_validate_proof_threshold_not_met() {
        let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

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
        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSignatures
            );
        })
    }
    #[test]
    fn fail_validate_proof_threshold_overflow() {
        let (env, mut signers, client) = setup_env(randint(0, 10), randint(1, 10));

        let last_index = signers.signers.signers.len() - 1;

        // get last signer and modify its weight to max u128 - 1
        if let Some(mut last_signer) = signers.signers.signers.get(last_index) {
            last_signer.weight = u128::MAX - 1;
            signers.signers.signers.set(last_index, last_signer);
        }

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let proof = generate_proof(&env, msg_hash.clone(), signers);

        // should panic, as modified signer wouldn't match the epoch
        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSignersHash
            );
        });
    }

    #[test]
    fn rotate_signers() {
        let (env, signers, client) = setup_env(1, randint(1, 10));

        let msg_hash: BytesN<32> = BytesN::random(&env);
        let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

        testutils::rotate_signers(&env, &client.address, new_signers.clone());

        let proof = generate_proof(&env, msg_hash.clone(), new_signers);

        env.as_contract(&client.address, || {
            assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        });
    }

    #[test]
    fn rotate_signers_fail_empty_signers() {
        let (env, _, _client) = setup_env(randint(0, 10), randint(1, 10));

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
        let (env, _, _client) = setup_env(1, randint(1, 10));

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
        let (env, _, _client) = setup_env(1, randint(1, 10));

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
        let (env, _, _client) = setup_env(1, randint(1, 10));
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
        let (env, _, _client) = setup_env(1, randint(1, 10));
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
        let (env, _, _client) = setup_env(1, randint(1, 10));

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
        let previous_signer_retention = randint(1, 5);
        let (env, original_signers, client) = setup_env(previous_signer_retention, randint(1, 10));

        let msg_hash: BytesN<32> = BytesN::random(&env);

        let mut previous_signers = original_signers.clone();

        for _ in 0..previous_signer_retention {
            let new_signers = generate_signers_set(
                &env,
                randint(1, 10),
                original_signers.domain_separator.clone(),
            );

            testutils::rotate_signers(&env, &client.address, new_signers.clone());

            let proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());

            env.as_contract(&client.address, || {
                assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
            });

            let proof = generate_proof(&env, msg_hash.clone(), previous_signers.clone());

            env.as_contract(&client.address, || {
                assert!(!assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
            });

            previous_signers = new_signers;
        }

        // Proof from the first signer set should still be valid
        let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());
        env.as_contract(&client.address, || {
            assert!(!assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        })
    }

    #[test]
    fn rotate_signers_panics_on_outdated_signer_set() {
        let previous_signer_retention = randint(0, 5);
        let (env, original_signers, client) = setup_env(previous_signer_retention, randint(1, 10));

        let msg_hash: BytesN<32> = BytesN::random(&env);

        for _ in 0..(previous_signer_retention + 1) {
            let new_signers = generate_signers_set(
                &env,
                randint(1, 10),
                original_signers.domain_separator.clone(),
            );
            testutils::rotate_signers(&env, &client.address, new_signers.clone());
        }

        // Proof from the first signer set should fail
        let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());

        env.as_contract(&client.address, || {
            assert_err!(
                auth::validate_proof(&env, &msg_hash, proof),
                ContractError::InvalidSigners
            )
        });
    }

    #[test]
    fn rotate_signers_fail_duplicated_signers() {
        let (env, signers, client) = setup_env(1, randint(1, 10));

        let msg_hash = BytesN::random(&env);
        let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);
        let duplicated_signers = new_signers.clone();

        testutils::rotate_signers(&env, &client.address, new_signers.clone());

        let proof = generate_proof(&env, msg_hash.clone(), new_signers);

        env.as_contract(&client.address, || {
            assert!(assert_ok!(auth::validate_proof(&env, &msg_hash, proof)));
        });

        // should panic, duplicated signers

        env.as_contract(&client.address, || {
            assert_err!(
                auth::rotate_signers(&env, &duplicated_signers.signers, false),
                ContractError::DuplicateSigners
            );
        });
    }
}
