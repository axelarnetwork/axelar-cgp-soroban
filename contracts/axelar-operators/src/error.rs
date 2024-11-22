use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractError {
    OperatorAlreadyAdded = 1,
    NotAnOperator = 2,
    /// Upgradeable
    MigrationNotAllowed = 3,
}
