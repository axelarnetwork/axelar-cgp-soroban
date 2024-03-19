use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    Owner,
    Operators(Address),
}
