use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Address, Map, Vec, crypto,
    serde::{Deserialize, Serialize}, xdr::Uint256
};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Axelar {
    // Auth Weighted
    crnt_epoch: u64, //current_epoch
    hash_epoch: Map<u64, BytesN<32>>, // hash_for_epoch
    epoch_hash: Map<BytesN<32>, u64>, // epoch_for_hash
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
            

        }//

    }

fn toSignedMsgHsh(
    hash: BytesN<32>
) {
    // return keccak256(abi.encodePacked('Soroban Signed Message:', hash));
    // can change prefix to whatever I want.
// can then use this for the validateProof & it wont have an impact as it's also made up on axelar side
}

fn validateProof() {

}

fn validateSig(
    hash: BytesN<32>,
    proof: Bytes
) {
    // what is ECDSA.recover()?

}