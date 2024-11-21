use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    /// Auth
    InvalidThreshold = 1,
    InvalidProof = 2,
    InvalidSigners = 3,
    InsufficientRotationDelay = 4,
    InvalidSignatures = 5,
    InvalidWeight = 6,
    WeightOverflow = 7,
    NotLatestSigners = 8,
    DuplicateSigners = 9,
    InvalidSignersHash = 10,
    InvalidEpoch = 11,
    /// Messages
    EmptyMessages = 12,
    /// Executable
    NotApproved = 13,
    /// Upgradeable
    MigrationNotAllowed = 14,
}
