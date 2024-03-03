use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    Uninitialized = 1,
    NoGatewaySet = 2,
    NotApproved = 3,
}
