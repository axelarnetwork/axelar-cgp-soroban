use soroban_sdk::{contracttype, Address};

#[derive(Clone)]
#[contracttype]
pub struct AllowanceDataKey {
    pub from: Address,
    pub spender: Address,
}

#[contracttype]
pub struct AllowanceValue {
    pub amount: i128,
    pub expiration_ledger: u32,
}

/// Do not use `METADATA_KEY`` as it is reserved for token metadata.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Allowance(AllowanceDataKey),
    Balance(Address),
    Minter(Address),
    TokenId,
    InterchainTokenService,
}
