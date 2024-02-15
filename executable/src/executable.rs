use soroban_sdk::{
    contractimpl, contracttype, contractclient, contracterror, bytes, panic_with_error,
    Bytes, BytesN, Env, Symbol, vec, Address, Map, map, Vec, crypto, bytesn, String,
    xdr::{self, FromXdr, ToXdr},
};

mod gateway {
    soroban_sdk::contractimport!(
        file = "../contract/target/wasm32-unknown-unknown/release/contract.wasm"
    );
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotApprovedByGateway = 8,
}

/// This is a trait that is implemented by the contract and provides a contract-specific way to execute a command.
#[contractclient(name = "ContractExecutableClient")]
pub trait ContractExecutable {
    fn _execute(env: Env, source_chain: String, source_address: String, payload: Bytes);
}

pub struct Executable;

#[contractimpl]
impl Executable {
    pub fn execute(
        env: Env,
        gateway_contract_id: BytesN<32>,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: String, // because soroban does not have msg.sender
        payload: Bytes,
    ) {
        let client = gateway::Client::new(&env, &gateway_contract_id);
        let payload_hash: BytesN<32> = env.crypto().keccak256(&payload);

        if (!client.validate_contract_call(&command_id, &source_chain, &source_address, &contract_address, &payload_hash)) {
            panic_with_error!(env, Error::NotApprovedByGateway);
        }

        let contract_id: BytesN<32> = env.call_stack().pop_back().unwrap().unwrap().0;
        let execute_client = ContractExecutableClient::new(&env, &contract_id);

        execute_client._execute(&source_chain, &source_address, &payload);
    }
}