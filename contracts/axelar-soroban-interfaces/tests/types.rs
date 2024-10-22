use hex_literal::hex;
use soroban_sdk::{xdr::ToXdr, Address, BytesN, Env, String, Vec};

use axelar_soroban_interfaces::types::{CommandType, Message, WeightedSigner, WeightedSigners};

#[test]
fn weighted_signers_hash() {
    let env = Env::default();

    let signers_array = [
        WeightedSigner {
            signer: BytesN::<32>::from_array(
                &env,
                &hex!("0a245a2a2a5e8ec439d1377579a08fc78ea55647ba6fcb1f5d8a360218e8a985"),
            ),
            weight: 3,
        },
        WeightedSigner {
            signer: BytesN::<32>::from_array(
                &env,
                &hex!("0b422cf449d900f6f8eb97f62e35811c62eb75feb84dfccef44a5c1c3dbac2ad"),
            ),
            weight: 2,
        },
        WeightedSigner {
            signer: BytesN::<32>::from_array(
                &env,
                &hex!("18c34bf01a11b5ba21ea11b1678f3035ef753f0bdb1d5014ec21037e8f99e2a2"),
            ),
            weight: 4,
        },
        WeightedSigner {
            signer: BytesN::<32>::from_array(
                &env,
                &hex!("f683ca8a6d7fe55f25599bb64b01edcc5eeb85fe5b63d3a4f0b3c32405005518"),
            ),
            weight: 4,
        },
        WeightedSigner {
            signer: BytesN::<32>::from_array(
                &env,
                &hex!("fbb4b870e800038f1379697fae3058938c59b696f38dd0fdf2659c0cf3a5b663"),
            ),
            weight: 2,
        },
    ];

    let weighted_signers = WeightedSigners {
        signers: Vec::from_array(&env, signers_array),
        threshold: 8u128,
        nonce: BytesN::<32>::from_array(
            &env,
            &hex!("8784bf7be5a9baaeea47e12d9e8ad0dec29afcbc3617d97f771e3c24fa945dce"),
        ),
    };

    let hash = weighted_signers.hash(&env).to_array();
    let signers_rotation_hash = weighted_signers.signers_rotation_hash(&env).to_array();

    goldie::assert_json!(vec![hex::encode(hash), hex::encode(signers_rotation_hash)]);
}

#[test]
fn messages_approval_hash() {
    let env = Env::default();

    let payload_hashes = [
        hex!("cfa347779c9b646ddf628c4da721976ceb998f1ab2c097b52e66a575c3975a6c"),
        hex!("fb5eb8245e3b8eb9d44f228ee142a3378f57d49fc95fa78d437ff8aa5dd564ba"),
        hex!("90e3761c0794fbbd8b563a0d05d83395e7f88f64f30eebb7c5533329f6653e84"),
        hex!("60e146cb9c548ba6e614a87910d8172c9d21279a3f8f4da256ff36e15b80ea30"),
    ]
    .map(|hash| BytesN::<32>::from_array(&env, &hash));

    let mut messages_array = soroban_sdk::Vec::new(&env);

    for (i, payload_hash) in payload_hashes.iter().enumerate() {
        messages_array.push_back(Message {
            source_chain: String::from_str(&env, &format!("source-{}", i + 1)),
            message_id: String::from_str(&env, &format!("test-{}", i + 1)),
            source_address: String::from_str(
                &env,
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAHK3M",
            ),
            contract_address: Address::from_string(&String::from_str(
                &env,
                "CAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMDR4",
            )),
            payload_hash: payload_hash.clone(),
        });
    }

    let approval_hash = env
        .crypto()
        .keccak256(&(CommandType::ApproveMessages, messages_array).to_xdr(&env))
        .to_array();

    goldie::assert!(hex::encode(approval_hash));
}
