use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Uninitialized = 1,
    EmptyMessages = 2,
    NotLatestSigners = 3,
    InvalidOperators = 4,
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AuthError {
    InvalidThreshold = 1,
    DuplicateOperators = 2,
    MalformedSigners = 3,
    LowSignaturesWeight = 4,
    InvalidProof = 5,
    InvalidSigners = 6,
    InsufficientRotationDelay = 7,
    InvalidSignatures = 8,
    InvalidWeights = 9,
    DuplicateSigners = 10,
}
