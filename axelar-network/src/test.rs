#![cfg(test)]

use super::*;
use soroban_sdk::{bytes, vec, Env};
extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);
    
}