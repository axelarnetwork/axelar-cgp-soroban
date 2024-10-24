use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    // General
    NotInitialized = 1,
    AlreadyInitialized = 2,
    // Auth
    InvalidThreshold = 3,
    InvalidProof = 4,
    InvalidSigners = 5,
    InsufficientRotationDelay = 6,
    InvalidSignatures = 7,
    InvalidWeight = 8,
    WeightOverflow = 9,
    NotLatestSigners = 10,
    DuplicateSigners = 11,
    // Messages
    EmptyMessages = 12,
}
