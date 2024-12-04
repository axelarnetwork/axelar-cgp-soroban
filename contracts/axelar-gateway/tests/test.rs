use axelar_gateway::error::ContractError;
use axelar_gateway::testutils::{
    generate_proof, generate_signers_set, generate_test_message, get_approve_hash, randint,
    setup_gateway, TestSignerSet,
};
use axelar_gateway::types::{
    Message, ProofSignature, ProofSigner, WeightedSigner, WeightedSigners,
};
use axelar_gateway::{AxelarGateway, AxelarGatewayClient};
use axelar_soroban_std::{
    assert_contract_err, assert_invocation, assert_invoke_auth_err, assert_invoke_auth_ok,
    assert_last_emitted_event,
};
use soroban_sdk::{
    bytes,
    testutils::{Address as _, BytesN as _, Events, MockAuth, MockAuthInvoke},
    vec, Address, BytesN, Env, String,
};
use soroban_sdk::{Symbol, Vec};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

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
fn call_contract() {
    let (env, _signers, client) = setup_env(1, 5);

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    client.call_contract(&user, &destination_chain, &destination_address, &payload);

    assert_invocation(
        &env,
        &user,
        &client.address,
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
        &client.address,
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
    let (env, _signers, client) = setup_env(1, 5);

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
        &client.address,
        "validate_message",
        (
            &contract_address,
            source_chain,
            message_id,
            source_address,
            payload_hash,
        ),
    );

    // there is an event emitted when initializing, ensure that no more are emitted
    assert_eq!(env.events().all().len(), 1);
}

#[test]
fn approve_message() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message(&env);
    let Message {
        source_chain,
        message_id,
        source_address,
        contract_address,
        payload_hash,
    } = message.clone();

    let messages = vec![&env, message.clone()];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, signers);
    client.approve_messages(&messages, &proof);

    assert_last_emitted_event(
        &env,
        &client.address,
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
        &client.address,
        (Symbol::new(&env, "message_executed"), message),
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
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message(&env);

    let invalid_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator);

    let messages = vec![&env, message];
    let data_hash = get_approve_hash(&env, messages.clone());
    let proof = generate_proof(&env, data_hash, invalid_signers);

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn approve_messages_fail_empty_messages() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

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
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let (message, _) = generate_test_message(&env);

    let messages = vec![&env, message];
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
    let (env, signers, client) = setup_env(1, 5);

    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = false;
    let new_epoch: u64 = client.epoch() + 1;

    client.rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay);

    assert_last_emitted_event(
        &env,
        &client.address,
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
        &client.address,
        (Symbol::new(&env, "message_approved"), message),
        (),
    );
}

#[test]
fn rotate_signers_bypass_rotation_delay() {
    let (env, signers, client) = setup_env(1, 5);
    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = true;
    let new_epoch: u64 = client.epoch() + 1;

    assert_invoke_auth_ok!(
        client.operator(),
        client.try_rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );

    assert_last_emitted_event(
        &env,
        &client.address,
        (
            Symbol::new(&env, "signers_rotated"),
            new_epoch,
            new_signers.signers.hash(&env),
        ),
        (),
    );
}

#[test]
fn rotate_signers_bypass_rotation_delay_unauthorized() {
    let (env, signers, client) = setup_env(1, 5);

    let new_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);
    let bypass_rotation_delay = true;

    assert_invoke_auth_err!(
        client.owner(),
        client.try_rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );

    let not_operator = Address::generate(&env);
    assert_invoke_auth_err!(
        not_operator,
        client.try_rotate_signers(&new_signers.signers, &proof, &bypass_rotation_delay)
    );
}

#[test]
fn rotate_signers_fail_not_latest_signers() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers.clone());
    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);

    let second_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = second_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    assert_contract_err!(
        client.try_rotate_signers(&second_signers.signers, &proof, &bypass_rotation_delay),
        ContractError::NotLatestSigners
    );
}

#[test]
fn transfer_operatorship() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    let operator = client.operator();
    let new_operator = Address::generate(&env);

    assert_invoke_auth_ok!(operator, client.try_transfer_operatorship(&new_operator));

    assert_last_emitted_event(
        &env,
        &client.address,
        (
            Symbol::new(&env, "operatorship_transferred"),
            operator,
            new_operator.clone(),
        ),
        (),
    );

    assert_eq!(client.operator(), new_operator);
}

#[test]
fn transfer_operatorship_unauthorized() {
    let (env, _, client) = setup_env(1, randint(1, 10));
    let not_operator = Address::generate(&env);

    assert_invoke_auth_err!(
        client.owner(),
        client.try_transfer_operatorship(&client.owner())
    );
    assert_invoke_auth_err!(
        not_operator,
        client.try_transfer_operatorship(&not_operator)
    );
}

#[test]
fn transfer_ownership() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    let owner = client.owner();
    let new_owner = Address::generate(&env);

    assert_invoke_auth_ok!(owner, client.try_transfer_ownership(&new_owner));
    assert_last_emitted_event(
        &env,
        &client.address,
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
fn transfer_ownership_unauthorized() {
    let (env, _, client) = setup_env(1, randint(1, 10));

    let new_owner = Address::generate(&env);

    assert_invoke_auth_err!(new_owner, client.try_transfer_ownership(&new_owner));
    assert_invoke_auth_err!(
        client.operator(),
        client.try_transfer_ownership(&client.operator())
    );
}

#[test]
fn epoch_by_signers_hash() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);

    assert_eq!(
        client.epoch_by_signers_hash(&first_signers.signers.hash(&env)),
        client.epoch()
    );
}

#[test]
fn epoch_by_signers_hash_fail_invalid_signers() {
    let (env, _, client) = setup_env(1, 5);
    let signers_hash = BytesN::<32>::from_array(&env, &[1; 32]);

    assert_contract_err!(
        client.try_epoch_by_signers_hash(&signers_hash),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn signers_hash_by_epoch() {
    let (env, signers, client) = setup_env(1, 5);

    let bypass_rotation_delay = false;

    let first_signers = generate_signers_set(&env, 5, signers.domain_separator.clone());
    let data_hash = first_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    client.rotate_signers(&first_signers.signers, &proof, &bypass_rotation_delay);
    let epoch = client.epoch();

    assert_eq!(
        client.signers_hash_by_epoch(&epoch),
        first_signers.signers.hash(&env)
    );
}

#[test]
fn signers_hash_by_epoch_fail_invalid_epoch() {
    let (_, _, client) = setup_env(1, 5);
    let invalid_epoch = 43u64;

    assert_contract_err!(
        client.try_signers_hash_by_epoch(&invalid_epoch),
        ContractError::InvalidEpoch
    );
}

#[test]
fn version() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    assert_eq!(
        client.version(),
        String::from_str(&env, env!("CARGO_PKG_VERSION"))
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Storage, MissingValue)")]
fn upgrade_invalid_wasm_hash() {
    let (env, _, client) = setup_env(1, randint(1, 10));

    let new_wasm_hash = BytesN::<32>::from_array(&env, &[0; 32]);
    // Should panic with invalid wasm hash
    client.upgrade(&new_wasm_hash);
}

#[test]
fn upgrade_unauthorized() {
    let (env, _signers, client) = setup_env(1, randint(1, 10));

    let not_owner = Address::generate(&env);
    let new_wasm_hash = BytesN::<32>::from_array(&env, &[0; 32]);

    assert_invoke_auth_err!(not_owner, client.try_upgrade(&new_wasm_hash));
    assert_invoke_auth_err!(client.operator(), client.try_upgrade(&new_wasm_hash));
}

#[test]
#[should_panic]
/// TODO: figure out how to detect error in constructor failure
fn fail_initialization_with_empty_signer_set() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let operator = Address::generate(&env);

    let empty_signer_set = Vec::<WeightedSigners>::new(&env);
    let domain_separator: BytesN<32> = BytesN::random(&env);
    let previous_signers_retention = randint(0, 10) as u64;
    let minimum_rotation_delay = 0;
    let initial_signers = empty_signer_set;

    // should panic because of empty signer set
    env.register(
        AxelarGateway,
        (
            owner,
            operator,
            domain_separator,
            minimum_rotation_delay,
            previous_signers_retention as u64,
            initial_signers,
        ),
    );
}

#[test]
#[should_panic(expected = "failed ED25519 verification")]
fn fail_validate_proof_invalid_signatures() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message.clone()];

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let proof = generate_proof(&env, msg_hash, signers);

    // should panic, proof is for different message hash
    // NOTE: panic occurs in std function, cannot handle explicitly
    client.approve_messages(&messages, &proof);
}

#[test]
fn fail_validate_proof_empty_signatures() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let mut proof = generate_proof(&env, msg_hash.clone(), signers.clone());

    // Modify signatures to make them invalid
    let mut new_signers = Vec::new(&env);
    for signer in proof.signers.iter() {
        new_signers.push_back(ProofSigner {
            signer: signer.signer,
            signature: ProofSignature::Unsigned,
        });
    }
    proof.signers = new_signers;

    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message];

    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSignatures
    );
}

#[test]
fn fail_validate_proof_threshold_not_met() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let mut total_weight = 0u128;

    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message.clone()];
    let msg_hash = get_approve_hash(&env, messages.clone());

    let mut proof = generate_proof(&env, msg_hash.clone(), signers);

    let mut new_signers = Vec::new(&env);
    for ProofSigner { signer, signature } in proof.signers {
        total_weight += signer.weight;

        let signature = if total_weight < proof.threshold {
            signature
        } else {
            ProofSignature::Unsigned
        };

        new_signers.push_back(ProofSigner { signer, signature });
    }
    proof.signers = new_signers;

    // should panic, all signatures are valid but total weight is below threshold
    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSignatures
    );
}

#[test]
fn fail_validate_proof_invalid_signer_set() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator.clone());

    let msg_hash: BytesN<32> = BytesN::random(&env);
    let invalid_proof = generate_proof(&env, msg_hash.clone(), new_signers.clone());
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &invalid_proof, &true),
        ContractError::InvalidSignersHash
    );
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
    let proof = generate_proof(&env, msg_hash.clone(), signers.clone());

    let (message, _) = generate_test_message(&env);
    let messages = vec![&env, message];

    // should panic, as modified signer wouldn't match the epoch
    assert_contract_err!(
        client.try_approve_messages(&messages, &proof),
        ContractError::InvalidSignersHash
    );
}

#[test]
fn rotate_signers_fail_empty_signers() {
    let (env, signers, client) = setup_env(randint(0, 10), randint(1, 10));

    let empty_signers = WeightedSigners {
        signers: Vec::<WeightedSigner>::new(&env),
        threshold: 0u128,
        nonce: BytesN::random(&env),
    };

    let data_hash = empty_signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // should throw an error, empty signer set
    assert_contract_err!(
        client.try_rotate_signers(&empty_signers, &proof, &true),
        ContractError::InvalidSigners
    );
}

#[test]
fn rotate_signers_fail_zero_weight() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    let last_index = new_signers.signers.signers.len() - 1;

    // get last signer and modify its weight to zero
    if let Some(mut last_signer) = new_signers.signers.signers.get(last_index) {
        last_signer.weight = 0u128;
        new_signers.signers.signers.set(last_index, last_signer);
    }

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // should throw an error, last signer weight is zero
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidWeight
    );
}

#[test]
fn rotate_signers_fail_weight_overflow() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    let mut new_signers = generate_signers_set(&env, randint(3, 10), BytesN::random(&env));

    let last_index = new_signers.signers.signers.len() - 1;

    // get last signer and modify its weight to max u128 - 1
    if let Some(mut last_signer) = new_signers.signers.signers.get(last_index) {
        last_signer.weight = u128::MAX - 1;
        new_signers.signers.signers.set(last_index, last_signer);
    }

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // last signer weight should cause overflow
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::WeightOverflow
    )
}

#[test]
fn rotate_signers_fail_zero_threshold() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
    let mut new_signers = generate_signers_set(&env, randint(1, 10), BytesN::random(&env));

    // set the threshold to zero
    new_signers.signers.threshold = 0u128;

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // should error because the threshold is set to zero
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidThreshold
    );
}

#[test]
fn rotate_signers_fail_low_total_weight() {
    let (env, signers, client) = setup_env(1, randint(1, 10));
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

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // should error because the threshold is set to zero
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidThreshold
    )
}

#[test]
fn rotate_signers_fail_wrong_signer_order() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

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

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash, signers);

    // should error because signers are in wrong order
    assert_contract_err!(
        client.try_rotate_signers(&new_signers.signers, &proof, &true),
        ContractError::InvalidSigners
    )
}

#[test]
fn rotate_signers_fail_duplicated_signers() {
    let (env, signers, client) = setup_env(1, randint(1, 10));

    // let msg_hash = BytesN::random(&env);
    let new_signers = generate_signers_set(&env, randint(1, 10), signers.domain_separator.clone());
    let duplicated_signers = new_signers.clone();

    let data_hash = new_signers.signers.signers_rotation_hash(&env);
    let proof = generate_proof(&env, data_hash.clone(), signers);
    client.rotate_signers(&new_signers.signers, &proof, &true);

    let proof = generate_proof(&env, data_hash, new_signers);

    // should panic, duplicated signers
    assert_contract_err!(
        client.try_rotate_signers(&duplicated_signers.signers, &proof, &true),
        ContractError::DuplicateSigners
    );
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
        let data_hash = new_signers.signers.signers_rotation_hash(&env);
        let proof = generate_proof(&env, data_hash, original_signers.clone());
        client.rotate_signers(&new_signers.signers, &proof, &true);
    }

    // Proof from the first signer set should fail
    let proof = generate_proof(&env, msg_hash.clone(), original_signers.clone());

    assert_contract_err!(
        client.try_rotate_signers(&original_signers.signers, &proof, &true),
        ContractError::InvalidSigners
    );
}
