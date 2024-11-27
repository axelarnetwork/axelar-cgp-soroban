use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Upgradeable
    MigrationNotAllowed = 1,
    /// Auth
    InvalidThreshold = 2,
    InvalidProof = 3,
    InvalidSigners = 4,
    InsufficientRotationDelay = 5,
    InvalidSignatures = 6,
    InvalidWeight = 7,
    WeightOverflow = 8,
    NotLatestSigners = 9,
    DuplicateSigners = 10,
    InvalidSignersHash = 11,
    InvalidEpoch = 12,
    /// Messages
    EmptyMessages = 13,
}
