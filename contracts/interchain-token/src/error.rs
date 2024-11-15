use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    AlreadyInitialized = 2,
    InvalidDecimal = 3,
    TokenIdZero = 4,
    TokenNameEmpty = 5,
    TokenSymbolEmpty = 6,
    NotAuthorizedMinter = 7,
}
