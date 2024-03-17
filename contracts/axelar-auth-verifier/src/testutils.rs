#![cfg(any(test, feature = "testutils"))]
extern crate std;

use crate::{
    contract::AxelarAuthVerifierClient,
    types::{Proof, WeightedSigners},
};
use rand::rngs::OsRng;
use rand::Rng;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use sha3::{Digest, Keccak256};
use soroban_sdk::{vec, U256};

use soroban_sdk::{symbol_short, xdr::ToXdr, Address, Bytes, BytesN, Env};

use axelar_soroban_std::types::Hash;
use axelar_soroban_std::{assert_emitted_event, traits::IntoVec};

#[derive(Clone, Debug)]
pub struct TestSignerSet {
    pub signers: std::vec::Vec<SecretKey>,
    pub signer_set: WeightedSigners,
}

pub fn randint(a: u32, b: u32) -> u32 {
    rand::thread_rng().gen_range(a..b)
}

pub fn generate_signer_set(env: &Env, num_signers: u32) -> TestSignerSet {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();

    let mut signer_keypair: std::vec::Vec<_> = (0..num_signers)
        .map(|_| {
            let sk = SecretKey::new(&mut OsRng);
            let pk = PublicKey::from_secret_key(&secp, &sk);
            let pk_hash: [u8; 32] = Keccak256::digest(pk.serialize_uncompressed()).into();
            let weight = rng.gen_range(1..10) as u32;

            (sk, (pk, pk_hash, weight))
        })
        .collect();

    // Sort signers by public key hash
    signer_keypair.sort_by(|(_, (_, h1, _)), (_, (_, h2, _))| h1.cmp(h2));

    let (signers, signer_info): (std::vec::Vec<_>, std::vec::Vec<(_, _, _)>) =
        signer_keypair.into_iter().unzip();
    let total_weight = signer_info.iter().map(|(_, _, w)| w).sum::<u32>();

    let signer_vec: std::vec::Vec<(Hash, U256)> = signer_info
        .into_iter()
        .map(|(_, pk_hash, w)| {
            (
                BytesN::<32>::from_array(env, &pk_hash),
                U256::from_u32(env, w),
            )
        })
        .collect();

    let threshold = rng.gen_range(1..=total_weight);

    let signer_set = WeightedSigners {
        signers: signer_vec.into_vec(env),
        threshold: U256::from_u32(env, threshold),
    };

    TestSignerSet {
        signers,
        signer_set,
    }
}

pub fn generate_proof(env: &Env, msg_hash: Hash, signers: TestSignerSet) -> Proof {
    let msg = Message::from_digest_slice(&msg_hash.to_array()).unwrap();
    let threshold = signers.signer_set.threshold.to_u128().unwrap() as u32;
    let secp = Secp256k1::new();

    let signatures: std::vec::Vec<_> = signers
        .signers
        .iter()
        .take(threshold as usize)
        .map(|signer| {
            let (recovery_id, signature) = secp
                .sign_ecdsa_recoverable(&msg, signer)
                .serialize_compact();

            (
                BytesN::<64>::from_array(env, &signature),
                recovery_id.to_i32() as u32,
            )
        })
        .collect();

    Proof {
        signer_set: signers.signer_set,
        signatures: signatures.into_vec(env),
    }
}

pub fn initialize(
    env: &Env,
    client: &AxelarAuthVerifierClient,
    owner: Address,
    previous_signer_retention: u32,
    num_signers: u32,
) -> TestSignerSet {
    let signers = generate_signer_set(env, num_signers);
    let signer_sets = vec![&env, signers.signer_set.clone()].to_xdr(env);
    let signer_set_hash = env
        .crypto()
        .keccak256(&signers.signer_set.clone().to_xdr(env));

    client.initialize(&owner, &previous_signer_retention, &signer_sets);

    assert_emitted_event(
        env,
        -1,
        &client.address,
        (symbol_short!("transfer"), signer_set_hash),
        (signers.signer_set.clone(),),
    );

    signers
}

pub fn transfer_operatorship(
    env: &Env,
    client: &AxelarAuthVerifierClient,
    new_signers: TestSignerSet,
) -> Bytes {
    let encoded_new_signer_set = new_signers.signer_set.clone().to_xdr(env);

    client.transfer_operatorship(&encoded_new_signer_set);

    assert_emitted_event(
        env,
        -1,
        &client.address,
        (
            symbol_short!("transfer"),
            env.crypto().keccak256(&encoded_new_signer_set),
        ),
        (new_signers.signer_set.clone(),),
    );

    encoded_new_signer_set
}
