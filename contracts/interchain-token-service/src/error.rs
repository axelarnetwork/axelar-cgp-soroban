use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    NotOwner = 1,
    TrustedAddressAlreadySet = 2,
    NoTrustedAddressSet = 3,
    /// Upgradeable
    MigrationNotAllowed = 4,
}
