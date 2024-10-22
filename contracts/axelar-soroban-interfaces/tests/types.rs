use hex_literal::hex;
use soroban_sdk::{BytesN, Env, Vec};

use axelar_soroban_interfaces::types::{WeightedSigner, WeightedSigners};

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
