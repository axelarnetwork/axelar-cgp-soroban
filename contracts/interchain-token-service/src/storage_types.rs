use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    TrustedAddress(String),
    Gateway,
    GasService,
    ChainName,
}
