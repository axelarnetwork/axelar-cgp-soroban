use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidOperators = 1,
    InvalidThreshold = 2,
    DuplicateOperators = 3,
    MalformedSigners = 4,
    LowSignaturesWeight = 5,
    InvalidProof = 6,
    OutdatedSigners = 7,
    InsufficientRotationDelay = 8,
}
