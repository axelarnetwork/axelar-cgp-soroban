use soroban_sdk::{contracttype, BytesN};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    PreviousSignerRetention,
    DomainSeparator,
    MinimumRotationDelay,
    Owner,
    Epoch,
    LastRotationTimestamp,
    SignerHashByEpoch(u64),
    EpochBySignerHash(BytesN<32>),
}
