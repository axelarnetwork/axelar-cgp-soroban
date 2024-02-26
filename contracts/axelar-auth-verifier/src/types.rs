use soroban_sdk::{contracttype, BytesN, Vec, U256};

#[contracttype]
#[derive(Clone, Debug)]
pub struct WeightedSigners {
    pub signers: Vec<(BytesN<32>, U256)>,
    pub threshold: U256,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Proof {
    pub signer_set: WeightedSigners,
    pub signatures: Vec<(BytesN<64>, u32)>,
}
