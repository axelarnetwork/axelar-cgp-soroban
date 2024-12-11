use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    NotOwner = 2,
    TrustedChainAlreadySet = 3,
    TrustedChainNotSet = 4,
    InvalidMessageType = 5,
    InvalidPayload = 6,
    UntrustedChain = 7,
    InsufficientMessageLength = 8,
    AbiDecodeFailed = 9,
    InvalidAmount = 10,
    InvalidUtf8 = 11,
    InvalidMinter = 12,
}
