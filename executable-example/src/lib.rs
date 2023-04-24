#![no_std]

use soroban_sdk::{contractimpl, contracttype, contractclient, contracterror, bytes, Bytes, BytesN, Env, Symbol, vec, Address, Map, map, Vec, crypto, bytesn,
    xdr::{self, FromXdr, ToXdr}, panic_with_error, String
};


mod axelar_executable {
    soroban_sdk::contractimport!(
        file = "../executable/target/wasm32-unknown-unknown/release/executable.wasm"
    );
}

 pub struct AxelarSorobanExample {
     // pub gateway_account_id: AccountId,
     // Example
     pub value: Option<String>,
     pub source_chain: Option<String>,
     pub source_address: Option<String>,
 }
 
 #[contractimpl]
 impl AxelarSorobanExample {
    //  pub fn new() -> Self {
    //      Self {
    //          value: None,
    //          source_chain: None,
    //          source_address: None,
    //      }
    //  }
 
    //  pub fn get_value(&self) -> Option<String> {
    //      self.value.clone()
    //  }
 
    //  pub fn get_source_chain(&self) -> Option<String> {
    //      self.source_chain.clone()
    //  }
 
    //  pub fn get_source_address(&self) -> Option<String> {
    //      self.source_address.clone()
    //  }
 
    //  pub fn set(&mut self, chain: String, destination_address: String, value: String) {
    //      self.value = Some(value.clone());
    //  }
 }
 
 impl ContractExecutable for AxelarSorobanExample {
     fn _execute(&mut self, source_chain: String, source_address: String, payload: Bytes) { 
         self.value = Some(payload);
         self.source_chain = Some(source_chain);
         self.source_address = Some(source_address);
     }
 }
 