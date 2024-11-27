use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    NotOwner = 2,
    TrustedAddressAlreadySet = 3,
    NoTrustedAddressSet = 4,
    InvalidMessageType = 5,
}
