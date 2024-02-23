use soroban_sdk::{contract, contractimpl, log, symbol_short, Address, Env, BytesN, Bytes, String, Symbol};

use crate::interface::AxelarGatewayInterface;

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn call_contract(env: Env, caller: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        caller.require_auth();

        // emit event
    }

    fn validate_contract_call(env: Env, caller: Address, command_id: BytesN<32>, source_chain: String, source_address: String, payload_hash: BytesN<32>) -> bool {
        // Implement the logic for validating a contract call.
        true
    }

    fn is_contract_call_approved(env: Env, command_id: BytesN<32>, source_chain: String, source_address: String, contract_address: Address, payload_hash: BytesN<32>) -> bool {
        // Implement the logic for checking if a contract call is approved.
        true
    }

    fn execute(env: Env, batch: Bytes) {
        // Implement the logic for executing a batch of commands.
    }
}

#[contractimpl]
impl AxelarGateway {
    fn approve_contract_call(env: Env, command_id: BytesN<32>, source_chain: String, source_address: String, contract_address: Address, payload_hash: BytesN<32>) {
        // Implement the logic for approving a contract call.
    }
}
