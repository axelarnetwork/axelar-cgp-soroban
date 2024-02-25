use soroban_sdk::xdr::FromXdr;
use soroban_sdk::{contract, contractimpl, log, symbol_short, Address, Env, BytesN, Bytes, String, Symbol};

use crate::interface::AxelarGatewayInterface;
use crate::types::{self, Command, ContractCallApproval, SignedCommandBatch};
use crate::{error::Error, event};
use crate::storage_types::{CommandExecutedKey, ContractCallApprovalKey, DataKey};

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

    fn execute(env: Env, batch: Bytes) -> Result<(), Error> {
        // Implement the logic for executing a batch of commands.
        // Err(Error::InvalidBatch)

        let signed_batch = SignedCommandBatch::from_xdr(&env, &batch).map_err(|_| Error::InvalidBatch)?;

        // validate proof

        let batch = signed_batch.batch;

        if batch.chain_id != 1 {
            return Err(Error::InvalidChainId);
        }

        for (command_id, command) in batch.commands {
            let key = Self::command_executed_key(command_id.clone());

            // Skip command if already executed. This allows batches to be processed partially.
            if env.storage().persistent().get(&key).unwrap_or(false) {
                continue
            }

            env.storage().persistent().set(&key, &true);

            match command {
                Command::ContractCallApproval(approval) => {
                    Self::approve_contract_call(&env, command_id.clone(), approval);
                }
                Command::TransferOperatorship(new_operators) => {
                    Self::transfer_operatorship(&env, new_operators);
                }
            }

            event::execute_command(&env, command_id);
        }

        Ok(())
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

    fn command_executed_key(command_id: BytesN<32>) -> DataKey {
        DataKey::CommandExecuted(CommandExecutedKey { command_id })
    }

    fn approve_contract_call(env: &Env, command_id: BytesN<32>, approval: types::ContractCallApproval) {
        let types::ContractCallApproval {
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        } = approval;

        let key = Self::contract_call_approval_key(command_id.clone(), source_chain.clone(), source_address.clone(), contract_address.clone(), payload_hash.clone());

        env.storage().persistent().set(&key, &true);

        event::approve_contract_call(env, command_id, source_chain, source_address, contract_address, payload_hash);
    }

    fn transfer_operatorship(env: &Env, new_operator: Bytes) {
        unimplemented!("Transfer operatorship");
    }
}
