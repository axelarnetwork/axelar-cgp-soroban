#![no_std]

use soroban_sdk::{contractimpl, contracttype, contractclient, contracterror, bytes, Bytes, BytesN, Env, Symbol, vec, Address, Map, map, Vec, crypto, bytesn,
    xdr::{self, FromXdr, ToXdr}, panic_with_error, String
};


mod axelar_executable {
    soroban_sdk::contractimport!(
        file = "../executable/target/wasm32-unknown-unknown/release/executable.wasm"
    );
}

mod gateway {
    soroban_sdk::contractimport!(
        file = "../contract/target/wasm32-unknown-unknown/release/contract.wasm"
    );
}

 pub struct AxelarSorobanExample;
 
 #[contractimpl]
 impl AxelarSorobanExample {
     pub fn get_value(env: Env) -> Bytes {
        env.storage().get(&Symbol::new(&env, &"value"))
     }
 
     pub fn get_source_chain(env: Env) -> String {
        env.storage().get(&Symbol::new(&env, &"source_chain"))
     }
 
     pub fn get_source_address(env: Env) -> String {
        env.storage().get(&Symbol::new(&env, &"source_address"))
     }
 
     pub fn set(env: Env, gateway_contract_id: BytesN<32>, caller: Address, chain: String, destination_address: String, payload: Bytes) {
        let client = gateway::Client::new(&env, &gateway_contract_id);
        client.call_contract(&caller, &chain, &destination_address, &payload);
     }
 }
 
 impl ContractExecutable for AxelarSorobanExample {
     fn _execute(env: Env, source_chain: String, source_address: String, payload: Bytes) { 
        env.storage().set(&Symbol::new(&env, &"value"), &payload);
        env.storage().set(&Symbol::new(&env, &"source_chain"), &source_chain);
        env.storage().set(&Symbol::new(&env, &"source_address"), &source_address);
     }
 }
 