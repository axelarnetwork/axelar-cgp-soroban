use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    Owner,
    TrustedAddress(String),
}
