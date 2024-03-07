use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String};

// use axelar_auth_verifier_contract::Client as AxelarAuthVerifierClient;
// use axelar_auth_verifier::interface::AxelarAuthVerifierInterface;
use axelar_auth_verifier::AxelarAuthVerifierClient;

use crate::interface::AxelarGatewayInterface;
use crate::storage_types::{ContractCallApprovalKey, DataKey};
use crate::types::{self, Command, SignedCommandBatch};
use crate::{error::Error, event};

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGateway {
    pub fn initialize_gateway(env: Env, auth_module: Address) {
        if env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
        {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage()
            .instance()
            .set(&DataKey::AuthModule, &auth_module);
    }
}

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) {
        caller.require_auth();

        let payload_hash = env.crypto().keccak256(&payload);

        event::call_contract(
            &env,
            caller,
            destination_chain,
            destination_address,
            payload,
            payload_hash,
        );
    }

    fn validate_contract_call(
        env: Env,
        caller: Address,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool {
        caller.require_auth();

        let key = Self::contract_call_approval_key(
            command_id.clone(),
            source_chain,
            source_address,
            caller,
            payload_hash,
        );

        let approved = env.storage().persistent().has(&key);

        if approved {
            env.storage().persistent().remove(&key);

            event::execute_contract_call(&env, command_id);
        }

        approved
    }

    fn is_contract_call_approved(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool {
        let key = Self::contract_call_approval_key(
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        );

        env.storage().persistent().has(&key)
    }

    fn execute(env: Env, batch: Bytes) -> Result<(), Error> {
        let SignedCommandBatch { batch, proof } =
            SignedCommandBatch::from_xdr(&env, &batch).map_err(|_| Error::InvalidBatch)?;
        let batch_hash = env.crypto().keccak256(&batch.clone().to_xdr(&env));

        let auth_module = AxelarAuthVerifierClient::new(
            &env,
            &env.storage().instance().get(&DataKey::AuthModule).unwrap(),
        );

        let valid = auth_module.validate_proof(&batch_hash, &proof);
        if !valid {
            return Err(Error::InvalidProof);
        }

        // TODO: switch to new domain separation approach
        if batch.chain_id != 1 {
            return Err(Error::InvalidChainId);
        }

        for (command_id, command) in batch.commands {
            let key = DataKey::CommandExecuted(command_id.clone());

            // TODO: switch to full revert, or add allow selecting subset of commands to process
            // Skip command if already executed. This allows batches to be processed partially.
            if env.storage().persistent().has(&key) {
                continue;
            }

            env.storage().persistent().set(&key, &true);

            match command {
                Command::ContractCallApproval(approval) => {
                    Self::approve_contract_call(&env, command_id.clone(), approval);
                }
                Command::TransferOperatorship(new_operators) => {
                    Self::transfer_operatorship(&env, &auth_module, new_operators);
                }
            }

            event::execute_command(&env, command_id);
        }

        Ok(())
    }
}

impl AxelarGateway {
    fn contract_call_approval_key(
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> DataKey {
        DataKey::ContractCallApproval(ContractCallApprovalKey {
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        })
    }

    fn approve_contract_call(
        env: &Env,
        command_id: BytesN<32>,
        approval: types::ContractCallApproval,
    ) {
        let types::ContractCallApproval {
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        } = approval;

        // TODO: further restrict contract_address value if needed (to avoid non applicable values that might be a valid Address)
        let key = Self::contract_call_approval_key(
            command_id.clone(),
            source_chain.clone(),
            source_address.clone(),
            contract_address.clone(),
            payload_hash.clone(),
        );

        env.storage().persistent().set(&key, &true);

        event::approve_contract_call(
            env,
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        );
    }

    fn transfer_operatorship(
        env: &Env,
        auth_module: &AxelarAuthVerifierClient,
        new_operator: Bytes,
    ) {
        auth_module.transfer_operatorship(&new_operator);

        event::transfer_operatorship(env, new_operator);
    }
}
