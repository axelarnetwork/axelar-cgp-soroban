use soroban_sdk::{contracttype, String};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    TrustedChain(String),
    Gateway,
    GasService,
    ItsHub,
    ChainName,
}
