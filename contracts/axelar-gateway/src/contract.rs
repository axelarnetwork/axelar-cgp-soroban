use soroban_sdk::{contract, contractimpl, log, symbol_short, Address, Env, BytesN, Bytes, String, Symbol};

use crate::interface::AxelarGatewayInterface;
use crate::event;
use crate::storage_types::{DataKey, ContractCallApprovalKey};

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn call_contract(env: Env, caller: Address, destination_chain: String, destination_address: String, payload: Bytes) {
        caller.require_auth();

        let payload_hash = env.crypto().keccak256(&payload);

        event::call_contract(&env, caller, destination_chain, destination_address, payload, payload_hash);
    }

    fn validate_contract_call(env: Env, caller: Address, command_id: BytesN<32>, source_chain: String, source_address: String, payload_hash: BytesN<32>) -> bool {
        caller.require_auth();

        let key = Self::contract_call_approval_key(command_id.clone(), source_chain, source_address, caller, payload_hash);

        let approved = env.storage().persistent().get(&key).unwrap_or(false);

        if approved {
            env.storage().persistent().set(&key, &false);

            event::execute_contract_call(&env, command_id);
        }

        approved
    }

    fn is_contract_call_approved(env: Env, command_id: BytesN<32>, source_chain: String, source_address: String, contract_address: Address, payload_hash: BytesN<32>) -> bool {
        let key = Self::contract_call_approval_key(command_id, source_chain, source_address, contract_address, payload_hash);

        env.storage().persistent().get(&key).unwrap_or(false)
    }

    fn execute(env: Env, batch: Bytes) {
        // Implement the logic for executing a batch of commands.
    }
}

impl AxelarGateway {
    fn contract_call_approval_key(command_id: BytesN<32>, source_chain: String, source_address: String, contract_address: Address, payload_hash: BytesN<32>) -> DataKey {
        DataKey::ContractCallApproval(ContractCallApprovalKey {
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        })
    }

    fn approve_contract_call(env: Env, command_id: BytesN<32>, source_chain: String, source_address: String, contract_address: Address, payload_hash: BytesN<32>) {
        // Implement the logic for approving a contract call.
    }
}
