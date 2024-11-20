use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Owner,
    TrustedAddress(String),
}
