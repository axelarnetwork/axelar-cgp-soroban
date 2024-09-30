use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Uninitialized = 1,
    EmptyMessages = 2,
    RotationAlreadyExecuted = 3,
    NotLatestSigners = 4,
    InvalidOperators = 5,
    InvalidThreshold = 6,
    DuplicateOperators = 7,
    MalformedSigners = 8,
    LowSignaturesWeight = 9,
    InvalidProof = 10,
    InvalidSigners = 11,
    InsufficientRotationDelay = 12,
    InvalidSignatures = 13,
    InvalidWeights = 14,
}
