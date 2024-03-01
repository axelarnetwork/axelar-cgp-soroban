use crate::{Incrementor2Contract, Incrementor2ContractClient, IncrementorContract, IncrementorContractClient};
use soroban_sdk::Env;

#[test]
fn increment() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IncrementorContract);
    let client = IncrementorContractClient::new(&env, &contract_id);

    assert_eq!(client.increment(), 1);
    assert_eq!(client.increment(), 2);
    assert_eq!(client.increment(), 3);
}

#[test]
fn increment_twice() {
    let env = Env::default();
    let contract_id = env.register_contract(None, IncrementorContract);
    let client = IncrementorContractClient::new(&env, &contract_id);

    let _ = env.register_contract(None, Incrementor2Contract);
    let client2 = Incrementor2ContractClient::new(&env, &contract_id);

    assert_eq!(client2.increment_twice(&client.address), 2);
}
