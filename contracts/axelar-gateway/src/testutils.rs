#![cfg(any(test, feature = "testutils"))]
extern crate std;

use crate::{contract::AxelarGatewayClient, types::CommandType};
use ed25519_dalek::{Signature, Signer, SigningKey};
use rand::Rng;

use soroban_sdk::{vec, String, testutils::BytesN as _, xdr::ToXdr, Bytes, BytesN, Env, Vec};
use soroban_sdk::{testutils::Address as _, Address};

use axelar_soroban_interfaces::types::{
    Message, Proof, ProofSignature, ProofSigner, WeightedSigner, WeightedSigners,
};

use axelar_soroban_std::traits::IntoVec;

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

#[derive(Clone, Debug)]
pub struct TestSignerSet {
    pub signers: std::vec::Vec<SigningKey>,
    pub signer_set: WeightedSigners,
    pub domain_separator: BytesN<32>,
}

pub fn initialize(
    env: &Env,
    client: &AxelarGatewayClient,
    operator: Address,
    previous_signer_retention: u32,
    num_signers: u32,
) -> TestSignerSet {
    let signers = generate_signer_set(env, num_signers, BytesN::random(env));
    let signer_sets = vec![&env, signers.signer_set.clone()];
    let minimum_rotation_delay = 0;

    client.initialize(
        &operator,
        &signers.domain_separator,
        &minimum_rotation_delay,
        &(previous_signer_retention as u64),
        &signer_sets,
    );

    signers
}

pub fn get_approve_hash(env: &Env, messages: Vec<Message>) -> BytesN<32> {
    env.crypto()
        .keccak256(&(CommandType::ApproveMessages, messages).to_xdr(env))
        .into()
}

pub fn get_rotation_hash(env: &Env, new_signers: WeightedSigners) -> BytesN<32> {
    env.crypto()
        .keccak256(&(CommandType::RotateSigners, new_signers).to_xdr(env))
        .into()
}

pub fn generate_test_message(env: &Env) -> (Message, Bytes) {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(0..20);
    let mut payload = std::vec![0u8; len];
    rng.fill(&mut payload[..]);

    let payload = Bytes::from_slice(env, &payload[..]);

    (
        Message {
            message_id: String::from_str(env, "test"),
            source_chain: String::from_str(env, DESTINATION_CHAIN),
            source_address: String::from_str(env, DESTINATION_ADDRESS),
            contract_address: Address::generate(env),
            payload_hash: env.crypto().keccak256(&payload).into(),
        },
        payload,
    )
}

pub fn randint(a: u32, b: u32) -> u32 {
    rand::thread_rng().gen_range(a..b)
}

pub fn generate_signer_set(
    env: &Env,
    num_signers: u32,
    domain_separator: BytesN<32>,
) -> TestSignerSet {
    let mut rng = rand::thread_rng();

    let mut signer_keypair: std::vec::Vec<_> = (0..num_signers)
        .map(|_| {
            let signing_key = SigningKey::generate(&mut rng);
            let weight = rng.gen_range(1..10) as u128;
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

    let threshold = rng.gen_range(1..=total_weight);

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
        .enumerate()
        .map(|(i, (signing_key, weighted_signer))| {
            if i > threshold {
                return ProofSigner {
                    signer: weighted_signer,
                    signature: ProofSignature::Unsigned,
                };
            }

            let signature: Signature = signing_key.sign(&msg_hash.to_array());
            ProofSigner {
                signer: weighted_signer,
                signature: ProofSignature::Signed(BytesN::<64>::from_array(
                    env,
                    &signature.to_bytes(),
                )),
            }
        })
        .collect();

    Proof {
        signers: proof_signers.into_vec(env),
        threshold: signers.signer_set.threshold,
        nonce: signers.signer_set.nonce,
    }
}
