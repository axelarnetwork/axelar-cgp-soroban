use soroban_sdk::{contractclient, contracterror, Address, Env, Symbol, Val, Vec};
/// Interface for the Axelar Operators contract.
#[contractclient(name = "AxelarOperators")]
pub trait AxelarOperatorsInterface {
    /// Initialize the operators contract with an owner.
    fn initialize(env: Env, owner: Address) -> Result<(), OperatorError>;

    /// Return true if the account is an operator.
    fn is_operator(env: Env, account: Address) -> bool;

    /// Add an address as an operator. Only callable by the contract owner.
    fn add_operator(env: Env, operator: Address) -> Result<(), OperatorError>;

    /// Remove an address as an operator. Only callable by the contract owner.
    fn remove_operator(env: Env, operator: Address) -> Result<(), OperatorError>;

    /// Execute a function on a contract as an operator.
    fn execute(
        env: Env,
        operator: Address,
        contract: Address,
        function_name: Symbol,
        args: Vec<Val>,
    ) -> Result<Val, OperatorError>;
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum OperatorError {
    OperatorAlreadyAdded = 1,
    NotAnOperator = 2,
    AlreadyInitialized = 3,
    ResourceNotFound = 4,
}
