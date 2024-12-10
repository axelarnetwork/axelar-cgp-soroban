use soroban_sdk::{contracttype, Address, Env, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Token {
    pub address: Address, // TODO: check if this can be changed to a TokenClient type instead which is richer than Address, or a generic type implementing TokenInterface
    pub amount: i128,
}

const ZERO_ADDRESS: &str = "0";

pub fn zero_adress(env: &Env) -> String {
    String::from_str(env, ZERO_ADDRESS)
}
