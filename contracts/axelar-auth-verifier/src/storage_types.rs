use soroban_sdk::{contracttype, BytesN};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    PreviousSignerRetention,
    Epoch,
    Owner,
    SignerHashByEpoch(u64),
    EpochBySignerHash(BytesN<32>),
}
