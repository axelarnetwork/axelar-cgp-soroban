use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum ContractError {
    InvalidDecimal = 1,
    TokenIdZero = 2,
    TokenNameEmpty = 3,
    TokenSymbolEmpty = 4,
    NotAuthorizedMinter = 5,
    InsufficientAllowance = 6,
}
