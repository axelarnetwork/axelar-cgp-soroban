use soroban_sdk::{contracterror, contractimpl, contracttype, bytes, Bytes, BytesN, Env, Address, Map, Vec, crypto,
    serde::{Deserialize, Serialize}, xdr::Uint256, symbol, Symbol, panic_with_error, bytesn
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
    pub operators: Vec<BytesN<32>>,
    pub weights: Vec<u128>, // uint256
    pub threshold: u128, // uint256
    pub signatures: Vec<(u32, BytesN<64>)>
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SignedMsg {
    pub text: Symbol,
    pub hash: BytesN<32>,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidOperators = 1,
    InvalidWeights = 2,
    InvalidThreshold = 3,
    DuplicateOperators = 4,
    LowSignaturesWeight = 5,
}

pub fn transfer_op( // transferOperatorship
    env: Env,
    params: Bytes
) -> bool {
    // IMPLEMENT REQUIRE OWNER?

    let tokens: Operatorship = Operatorship::deserialize(&env, &params).unwrap();
    let new_operators: Vec<BytesN<32>> = tokens.new_ops;
    let new_weights: Vec<u128> = tokens.new_wghts;
    let new_threshold: u128 = tokens.new_thres;
    
    let operators_length: u32 = new_operators.len();
    let weights_length: u32 = new_weights.len();

    if operators_length == 0 || is_sorted_asc_no_dup(env.clone(), new_operators.clone())// implement 2nd condition
    {
        panic_with_error!(env, Error::InvalidOperators);

    }

    if weights_length != operators_length {
        panic_with_error!(env, Error::InvalidWeights);
    }

    let mut total_weight: u128 = 0;

    for i in 0..weights_length {
        total_weight += new_weights.get(i).unwrap().unwrap();
    }

    if new_threshold == 0 || total_weight < new_threshold {
        panic_with_error!(env, Error::InvalidThreshold);
    }

    let new_operators_hash: BytesN<32> = env.crypto().sha256(&params);
    // create function that adds a prefix to new_operators_hash?
    
    let existing_epoch: u64 = env.storage().get::<&soroban_sdk::BytesN<32>, u64>(&new_operators_hash).unwrap().unwrap_or(0);
    
    if existing_epoch > 0 {
        //implementation: make variables all in one big hash, but the hash for epoch map is prefixed.
        panic_with_error!(env, Error::DuplicateOperators);
    }

    let epoch: u64 = env.storage().get::<soroban_sdk::Symbol, u64>(symbol!("cur_epoch")).unwrap().unwrap() + 1;
    env.storage().set(&symbol!("cur_epoch"), &epoch);
    env.storage().set(&epoch, &new_operators_hash);
    env.storage().set(&new_operators_hash, &epoch);

    let event: Operatorship = Operatorship { new_ops: new_operators, new_wghts: new_weights, new_thres: new_threshold};
    env.events().publish((), event);

    return true;

}

pub fn to_signed_msg_hsh(
    env: Env,
    hash: BytesN<32>
) -> BytesN<32> {
    let data: SignedMsg = SignedMsg {
        text: symbol!("Soroban"),
        hash: hash
    };
    return env.crypto().sha256(&data.serialize(&env));
    // return keccak256(abi.encodePacked('Soroban Signed Message:', hash));
    // can change prefix to whatever I want.
// can then use this for the validateProof & it wont have an impact as it's also made up on axelar side
}

pub fn validate_proof(
    env: Env,
    msghash: BytesN<32>,
    proof: Bytes
) -> bool {
    let tokens: Validate = Validate::deserialize(&env, &proof).unwrap();
    let operators: Vec<BytesN<32>> = tokens.operators;
    let weights: Vec<u128> = tokens.weights;
    let threshold: u128 = tokens.threshold;
    let signatures: Vec<(u32, BytesN<64>)> = tokens.signatures;

    let operator: Operatorship = Operatorship {
        new_ops: operators.clone(),
        new_wghts: weights.clone(),
        new_thres: threshold
    };
    let operators_hash: BytesN<32> = env.crypto().sha256(&operator.serialize(&env));

    let operators_epoch: u128 = env.storage().get(operators_hash).unwrap_or(Ok(0)).unwrap(); //uint256
    let epoch: u128 = env.storage().get(symbol!("cur_epoch")).unwrap_or(Ok(0)).unwrap(); //uint256

    if (operators_epoch == 0 || epoch - operators_epoch >= 16) {
        panic_with_error!(env, Error::InvalidOperators);
    }

    validate_sig(env, msghash, operators, weights, threshold, signatures);

    return operators_epoch == epoch;
    

}

fn validate_sig(
    env: Env,
    msghash: BytesN<32>,
    public_keys: Vec<BytesN<32>>, // operators
    weights: Vec<u128>, //uint256
    threshold: u128, //uint256
    signatures: Vec<(u32, BytesN<64>)> 

) {
    // CHANGE ALL u128 TO UINT256
    let mut weight: u128 = 0;
    let msg_hash: Bytes = msghash.into(); // convert into Bytes

    for i in 0..signatures.len() {
        let public_key_idx: u32 = signatures.get(i).unwrap().unwrap().0;
        
        env.crypto().ed25519_verify(
            &public_keys.get(public_key_idx).unwrap().unwrap(), 
            &msg_hash, 
            &signatures.get(i).unwrap().unwrap().1);
   
        // return if weight sum above threshold
        weight += weights.get(public_key_idx).unwrap().unwrap();
        // weight needs to reach or surpass threshold
        if (weight >= threshold) {
            return; 
        }
    }
    // if weight sum below threshold
    panic_with_error!(env, Error::LowSignaturesWeight);

}

fn is_sorted_asc_no_dup(
    env: Env,
    accounts: Vec<BytesN<32>>
) -> bool {
    for i in 0..accounts.len()-1 {
        if accounts.get(i).unwrap().unwrap() >= accounts.get(i+1).unwrap().unwrap() {
            return false;
        }
    }

    // can the 0th index of accounts even equal the "0 address" in the rhs?
    return accounts.get(0).unwrap().unwrap() != bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000000);
}