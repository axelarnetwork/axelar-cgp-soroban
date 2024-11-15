use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    AlreadyInitialized = 1,
    InvalidDecimal = 2,
    TokenIdZero = 3,
    TokenNameEmpty = 4,
    TokenSymbolEmpty = 5,
    NotAuthorizedMinter = 6,
}
