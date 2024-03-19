use axelar_soroban_std::types::Hash;
use soroban_sdk::contracttype;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    PreviousSignerRetention,
    Epoch,
    Owner,
    SignerHashByEpoch(u64),
    EpochBySignerHash(Hash),
}
