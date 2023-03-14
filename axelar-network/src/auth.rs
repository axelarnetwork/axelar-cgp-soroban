use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Address, Map, Vec, crypto,
    serde::{Deserialize, Serialize}, xdr::Uint256, symbol
};

use crate::Operatorship;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Axelar {
    // Auth Weighted
    crnt_epoch: u64, //current_epoch
    hash_epoch: Map<u64, BytesN<32>>, // hash_for_epoch
    epoch_hash: Map<BytesN<32>, u64>, // epoch_for_hash
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Validate {
    pub operators: Vec<Address>,
    pub weights: Vec<u128>, // uint256
    pub threshold: u128, // uint256
    pub signatures: Vec<Bytes>
}

    pub fn transferOp( // transferOperatorship
        env: Env,
        params: Bytes
    ) {
        let tokens: Operatorship = Operatorship::deserialize(&env, &params).unwrap();
        let new_operators: Vec<Address> = tokens.new_ops;
        let new_weights: Vec<u128> = tokens.new_wghts;
        let new_threshold: u128 = tokens.new_thres;
        
        let operators_length: u32 = new_operators.len();
        let weights_length: u32 = new_weights.len();

        if operators_length == 0
        {
            // implement
        }

        if weights_length != operators_length {
            // implement
        }

        let mut total_weight: u128 = 0;

        for i in 0..weights_length {
            total_weight += new_weights.get(i).unwrap().unwrap();
        }

        if new_threshold == 0 || total_weight < new_threshold {
            // implement
        }

        let new_operators_hash: BytesN<32> = env.crypto().sha256(&params);
        // create function that adds a prefix to new_operators_hash.

        //if env.storage().get(new_operators_hash) > 0 {
            //implementation: make variables all in one big hash, but the hash for epoch map is prefixed.
            

        //}

    }

fn toSignedMsgHsh(
    hash: BytesN<32>
) {
    // return keccak256(abi.encodePacked('Soroban Signed Message:', hash));
    // can change prefix to whatever I want.
// can then use this for the validateProof & it wont have an impact as it's also made up on axelar side
}

fn validate_proof(
    env: Env,
    msghash: BytesN<32>,
    proof: Bytes
) -> bool {
    let tokens: Validate = Validate::deserialize(&env, &proof).unwrap();
    let operators: Vec<Address> = tokens.operators;
    let weights: Vec<u128> = tokens.weights;
    let threshold: u128 = tokens.threshold;
    let signatures: Vec<Bytes> = tokens.signatures;

    let operator: Operatorship = Operatorship {
        new_ops: operators.clone(),
        new_wghts: weights.clone(),
        new_thres: threshold
    };
    let operators_hash: BytesN<32> = env.crypto().sha256(&operator.serialize(&env));

    let operators_epoch: u128 = env.storage().get(operators_hash).unwrap().unwrap(); //uint256
    let epoch: u128 = env.storage().get(symbol!("cur_epoch")).unwrap().unwrap(); //uint256

    if (operators_epoch == 0 || epoch - operators_epoch >= 16) {
        // implement
    }

    validateSig(env, msghash, operators, weights, threshold, signatures);

    return operators_epoch == epoch;
    

}

fn validateSig(
    env: Env,
    msghash: BytesN<32>,
    operators: Vec<Address>,
    weights: Vec<u128>, //uint256
    threshold: u128, //uint256
    signatures: Vec<Bytes> 
) {
    // what is ECDSA.recover()?

}