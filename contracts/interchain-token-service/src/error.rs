use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotOwner = 3,
    TrustedAddressAlreadySet = 4,
    NoTrustedAddressSet = 5,
}
