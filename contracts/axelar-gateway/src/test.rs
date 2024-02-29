#![cfg(test)]
extern crate std;

mod axelar_auth_verifier_contract {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/axelar_auth_verifier.wasm"
    );
}

use axelar_soroban_std::traits::IntoVec;
use axelar_auth_verifier::types::{Proof, WeightedSigners};
use rand::rngs::OsRng;
use rand::Rng;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use soroban_sdk::{TryFromVal, U256};
use sha3::{Digest, Keccak256};

use crate::types::{self, CommandBatch, SignedCommandBatch};
use crate::{contract::AxelarGateway, AxelarGatewayClient};
use soroban_sdk::{
    bytes, symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _, Events},
    vec,
    xdr::ToXdr,
    Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Val, Vec,
};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn setup_env<'a>() -> (Env, Address, AxelarGatewayClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    (env, contract_id, client)
}

fn generate_signer_set(env: &Env, num_signers: u32) -> (std::vec::Vec<SecretKey>, WeightedSigners) {
    let secp = Secp256k1::new();
    let mut rng = rand::thread_rng();

    let mut signer_keypair: std::vec::Vec<_> = (0..num_signers).map(|_| {
        let sk = SecretKey::new(&mut OsRng);
        let pk = PublicKey::from_secret_key(&secp, &sk);
        let pk_hash: [u8; 32] = Keccak256::digest(pk.serialize_uncompressed()).into();
        let weight = rng.gen_range(1..10) as u32;

        (sk, (pk, pk_hash, weight))
    }).collect();

    signer_keypair.sort_by(|(_, (_, h1, _)), (_, (_, h2, _))| h1.cmp(h2));

    let (signers, signer_info): (std::vec::Vec<_>, std::vec::Vec<(_, _, _)>) = signer_keypair.into_iter().unzip();
    let total_weight = signer_info.iter().map(|(_, _, w)| w).sum::<u32>();

    let signer_vec: std::vec::Vec<(BytesN<32>, U256)> = signer_info.into_iter().map(|(_, pk_hash, w)| {
        (BytesN::<32>::from_array(env, &pk_hash), U256::from_u32(env, w))
    }).collect();

    let threshold = rng.gen_range(1..=total_weight);

    let signer_set = WeightedSigners {
        signers: signer_vec.into_vec(env),
        threshold: U256::from_u32(env, threshold),
    };

    (signers, signer_set)
}

fn generate_proof(env: &Env, batch: CommandBatch, signers: std::vec::Vec<SecretKey>, signer_set: WeightedSigners) -> SignedCommandBatch {
    let msg_hash = env.crypto().keccak256(&batch.clone().to_xdr(env));
    let msg = Message::from_digest_slice(&msg_hash.to_array()).unwrap();
    let threshold = signer_set.threshold.to_u128().unwrap() as u32;
    let secp = Secp256k1::new();

    let signatures: std::vec::Vec<_> = signers.iter().take(threshold as usize).map(|signer| {
        let (recovery_id, signature) = secp.sign_ecdsa_recoverable(&msg, signer).serialize_compact();

        (BytesN::<64>::from_array(env, &signature), recovery_id.to_i32() as u32)
    }).collect();

    let proof = Proof { signer_set, signatures: signatures.into_vec(env) }.to_xdr(env);

    SignedCommandBatch{ batch, proof }
}

fn generate_test_approval(env: &Env) -> (types::ContractCallApproval, Bytes) {
    let payload = bytes!(&env, 0x1234);

    (
        types::ContractCallApproval {
            source_chain: String::from_str(env, DESTINATION_CHAIN),
            source_address: String::from_str(env, DESTINATION_ADDRESS),
            contract_address: Address::generate(env),
            payload_hash: env.crypto().keccak256(&payload),
        },
        payload,
    )
}

fn assert_invocation(
    env: &Env,
    caller: &Address,
    contract_id: &Address,
    function_name: &str,
    args: Vec<Val>,
) {
    assert_eq!(
        env.auths(),
        std::vec![(
            caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(env, function_name),
                    args,
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
fn assert_emitted_event<U, V>(
    env: &Env,
    event_index: u32,
    contract_id: &Address,
    topics: U,
    data: V,
) where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    let events = env.events().all();
    assert!(event_index < events.len(), "event_index out of bounds");

    let event = events.get(event_index).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}

#[test]
fn call_contract() {
    let (env, contract_id, client) = setup_env();

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    client.call_contract(&user, &destination_chain, &destination_address, &payload);

    assert_invocation(
        &env,
        &user,
        &contract_id,
        "call_contract",
        (
            &user,
            destination_chain.clone(),
            destination_address.clone(),
            payload.clone(),
        )
            .into_val(&env),
    );

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (
            symbol_short!("called"),
            user,
            env.crypto().keccak256(&payload),
        ),
        (destination_chain, destination_address, payload),
    );
}

#[test]
fn validate_contract_call() {
    let (env, contract_id, client) = setup_env();

    let (
        types::ContractCallApproval {
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        },
        _,
    ) = generate_test_approval(&env);

    let command_id = BytesN::random(&env);

    let approved = client.validate_contract_call(
        &contract_address,
        &command_id,
        &source_chain,
        &source_address,
        &payload_hash,
    );
    assert!(!approved);

    assert_invocation(
        &env,
        &contract_address,
        &contract_id,
        "validate_contract_call",
        (
            &contract_address,
            command_id.clone(),
            source_chain.clone(),
            source_address.clone(),
            payload_hash.clone(),
        )
            .into_val(&env),
    );

    assert_eq!(env.events().all().len(), 0);
}

#[test]
fn approve_contract_call() {
    let (env, contract_id, client) = setup_env();
    let (approval, _) = generate_test_approval(&env);
    let types::ContractCallApproval {
        source_chain,
        source_address,
        contract_address,
        payload_hash,
    } = approval.clone();
    let command_id = BytesN::random(&env);
    let (signers, signer_set) = generate_signer_set(&env, 3);
    let signer_sets = vec![&env, signer_set.clone()].to_xdr(&env);

    let auth_contract_id = env.register_contract_wasm(None, axelar_auth_verifier_contract::WASM);
    let auth_client = axelar_auth_verifier_contract::Client::new(&env, &auth_contract_id);
    auth_client.initialize(&2, &signer_sets);

    client.initialize(&auth_contract_id);

    let batch = types::CommandBatch {
        chain_id: 1,
        commands: vec![
            &env,
            (
                command_id.clone(),
                types::Command::ContractCallApproval(approval),
            ),
        ],
    };

    let signed_batch = generate_proof(&env, batch, signers, signer_set);

    client.execute(&signed_batch.to_xdr(&env));

    assert_emitted_event(
        &env,
        0,
        &contract_id,
        (
            symbol_short!("approved"),
            command_id.clone(),
            contract_address,
            payload_hash,
        ),
        (source_chain, source_address),
    );

    assert_emitted_event(
        &env,
        1,
        &contract_id,
        (symbol_short!("command"), command_id),
        (),
    );
}
