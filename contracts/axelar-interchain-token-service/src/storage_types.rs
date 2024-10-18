use soroban_sdk::{contracttype, BytesN, String};

#[contracttype]
#[derive(Clone, Debug)]
pub struct TrustedItsAddress {
    pub chain_name: String,
    pub trusted_address: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    AuthModule,
    Owner,
    TrustedAddress(TrustedItsAddress),
}