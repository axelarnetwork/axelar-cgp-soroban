use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    InvalidBatch = 1,
    InvalidChainId = 2,
    CommandAlreadyExecuted = 3,
    InvalidProof = 4,
}

