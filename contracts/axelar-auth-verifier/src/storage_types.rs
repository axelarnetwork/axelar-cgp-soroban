use axelar_soroban_std::types::Hash;
use soroban_sdk::contracttype;

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
    EpochBySignerHash(Hash),
}
