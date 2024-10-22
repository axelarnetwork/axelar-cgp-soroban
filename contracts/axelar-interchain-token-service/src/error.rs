use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum InterchainTokenServiceError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotOwner = 3,
    TrustedAddressAlreadyAdded = 4,
    NotTrustedAddress = 5,
}