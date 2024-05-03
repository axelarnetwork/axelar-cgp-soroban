use soroban_sdk::{contractclient, Address, Env, Symbol, Val, Vec};

/// Interface for the Axelar Operators contract.
#[contractclient(name = "AxelarOperators")]
pub trait AxelarOperatorsInterface {
    /// Initialize the operators contract with an owner.
    fn initialize(env: Env, owner: Address);

    /// Return true if the account is an operator.
    fn is_operator(env: Env, account: Address) -> bool;

    /// Add an address as an operator. Only callable by the contract owner.
    fn add_operator(env: Env, operator: Address);

    /// Remove an address as an operator. Only callable by the contract owner.
    fn remove_operator(env: Env, operator: Address);

    /// Execute a function on a contract as an operator.
    fn execute(env: Env, operator: Address, contract: Address, func: Symbol, args: Vec<Val>)
        -> Val;
}
