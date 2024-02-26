use soroban_sdk::{contracttype, Address, BytesN, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    PreviousSignerRetention,
    Epoch,
    SignerHashByEpoch(u64),
    EpochBySignerHash(BytesN<32>),
}
