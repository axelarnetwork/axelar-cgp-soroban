use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Uninitialized = 1,
    NotApproved = 2,
    InvalidCommand = 3,
    ProposalNotFound = 4,
    ProposalNotReady = 5,
}
