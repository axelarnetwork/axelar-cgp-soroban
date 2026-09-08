use stellar_axelar_std::{contracterror, soroban_sdk};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
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
    EmptySigners = 13,
    OutdatedSigners = 14,
    EmptyMessages = 15,
    ContractPaused = 16,
    InvalidMessageApproval = 17,
    MigrationInProgress = 18,
    AlreadyPaused = 19,
    NotPaused = 20,
}
