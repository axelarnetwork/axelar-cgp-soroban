use soroban_sdk::{contracttype, Address, BytesN};

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
