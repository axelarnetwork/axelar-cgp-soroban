use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    InvalidDecimal = 2,
    InvalidTokenName = 3,
    InvalidTokenSymbol = 4,
    NotMinter = 5,
}
