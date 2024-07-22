#![cfg(any(test, feature = "testutils"))]
extern crate std;

use crate::contract::AxelarAuthVerifierClient;
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::rngs::OsRng;
use rand::Rng;
use soroban_sdk::vec;
use soroban_sdk::{symbol_short, testutils::BytesN as _, xdr::ToXdr, Address, Bytes, BytesN, Env};

use axelar_soroban_interfaces::types::{Proof, ProofSigner, WeightedSigner, WeightedSigners};
use axelar_soroban_std::{assert_emitted_event, traits::IntoVec};

#[derive(Clone, Debug)]
pub struct TestSignerSet {
    pub signers: std::vec::Vec<SigningKey>,
    pub signer_set: WeightedSigners,
    pub domain_separator: BytesN<32>,
}

pub fn randint(a: u32, b: u32) -> u32 {
    rand::thread_rng().gen_range(a..b)
}

pub fn generate_random_payload_and_hash(env: &Env) -> BytesN<32> {
    let payload: Bytes = BytesN::<10>::random(env).into();
    env.crypto().keccak256(&payload).into()
}

pub fn generate_signer_set(
    env: &Env,
    num_signers: u32,
    domain_separator: BytesN<32>,
) -> TestSignerSet {
    let mut csprng = OsRng {};

    let mut signer_keypair: std::vec::Vec<_> = (0..num_signers)
        .map(|_| {
            let signing_key = SigningKey::generate(&mut csprng);
            let weight = csprng.gen_range(1..10) as u128;
            (signing_key, weight)
        })
        .collect();

    // Sort signers by public key
    signer_keypair.sort_by(|a, b| {
        a.0.verifying_key()
            .to_bytes()
            .cmp(&b.0.verifying_key().to_bytes())
    });

    let total_weight = signer_keypair.iter().map(|(_, w)| w).sum::<u128>();

    let signer_vec: std::vec::Vec<WeightedSigner> = signer_keypair
        .iter()
        .map(|(signing_key, w)| WeightedSigner {
            signer: BytesN::<32>::from_array(env, &signing_key.verifying_key().to_bytes()),
            weight: *w,
        })
        .collect();

    let threshold = csprng.gen_range(1..=total_weight);

    let signer_set = WeightedSigners {
        signers: signer_vec.into_vec(env),
        threshold,
        nonce: BytesN::<32>::from_array(env, &[0; 32]),
    };

    TestSignerSet {
        signers: signer_keypair
            .into_iter()
            .map(|(signing_key, _)| signing_key)
            .collect(),
        signer_set,
        domain_separator,
    }
}

pub fn generate_proof(env: &Env, data_hash: BytesN<32>, signers: TestSignerSet) -> Proof {
    let signer_hash = env
        .crypto()
        .keccak256(&signers.signer_set.clone().to_xdr(env));

    let mut msg: Bytes = signers.domain_separator.into();
    msg.extend_from_array(&signer_hash.to_array());
    msg.extend_from_array(&data_hash.to_array());
    let msg_hash = env.crypto().keccak256(&msg);

    let threshold = signers.signer_set.threshold as usize;

    let proof_signers: std::vec::Vec<_> = signers
        .signers
        .iter()
        .zip(signers.signer_set.signers.iter())
        .take(threshold)
        .map(|(signing_key, weighted_signer)| {
            let signature: Signature = signing_key.sign(&msg_hash.to_array());
            ProofSigner {
                signer: BytesN::<32>::from_array(env, &signing_key.verifying_key().to_bytes()),
                weight: weighted_signer.weight.clone(),
                signature: Bytes::from_array(env, &signature.to_bytes()),
            }
        })
        .collect();

    Proof {
        signers: proof_signers.into_vec(env),
        threshold: signers.signer_set.threshold,
        nonce: signers.signer_set.nonce,
    }
}

pub fn initialize(
    env: &Env,
    client: &AxelarAuthVerifierClient,
    owner: Address,
    previous_signer_retention: u32,
    num_signers: u32,
) -> TestSignerSet {
    let signers = generate_signer_set(env, num_signers, BytesN::random(env));
    let signer_sets = vec![&env, signers.signer_set.clone()];
    let signer_set_hash = env
        .crypto()
        .keccak256(&signers.signer_set.clone().to_xdr(env));
    let minimum_rotation_delay = 0;

    client.initialize(
        &owner,
        &(previous_signer_retention as u64),
        &signers.domain_separator,
        &minimum_rotation_delay,
        &signer_sets,
    );

    assert_emitted_event(
        env,
        -1,
        &client.address,
        (symbol_short!("rotated"), 1u64, signer_set_hash),
        (signers.signer_set.clone(),),
    );

    signers
}

pub fn rotate_signers(env: &Env, client: &AxelarAuthVerifierClient, new_signers: TestSignerSet) {
    let encoded_new_signer_set = new_signers.signer_set.clone().to_xdr(env);

    let epoch: u64 = client.epoch() + 1;

    client.rotate_signers(&new_signers.signer_set, &false);

    assert_emitted_event(
        env,
        -1,
        &client.address,
        (
            symbol_short!("rotated"),
            epoch,
            env.crypto().keccak256(&encoded_new_signer_set),
        ),
        (new_signers.signer_set.clone(),),
    );
}
