use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, Env, String};

use axelar_soroban_interfaces::axelar_auth_verifier::AxelarAuthVerifierClient;
use axelar_soroban_std::types::Hash;

use crate::storage_types::{ContractCallApprovalKey, DataKey};
use crate::types::{self, Command, SignedCommandBatch};
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_gateway::AxelarGatewayInterface;

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn initialize(env: Env, auth_module: Address) {
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
        command_id: Hash,
        source_chain: String,
        source_address: String,
        payload_hash: Hash,
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
        command_id: Hash,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: Hash,
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

    fn execute(env: Env, batch: Bytes) {
        let SignedCommandBatch { batch, proof } = match SignedCommandBatch::from_xdr(&env, &batch) {
            Ok(x) => x,
            Err(_) => panic_with_error!(env, Error::InvalidBatch),
        };
        let batch_hash = env.crypto().keccak256(&batch.clone().to_xdr(&env));

        let auth_module = AxelarAuthVerifierClient::new(
            &env,
            match &env.storage().instance().get(&DataKey::AuthModule) {
                Some(auth) => auth,
                None => panic_with_error!(env, Error::Uninitialized),
            },
        );

        let valid = auth_module.validate_proof(&batch_hash, &proof);
        if !valid {
            panic_with_error!(env, Error::InvalidProof);
        }

        // TODO: switch to new domain separation approach
        if batch.chain_id != 1 {
            panic_with_error!(env, Error::InvalidChainId);
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
    }

    fn approve_messages(env: Env, messages: soroban_sdk::Vec<axelar_soroban_interfaces::types::Message>, proof: axelar_soroban_interfaces::types::Proof) {

    }

    fn rotate_signers(env: Env, signers: axelar_soroban_interfaces::types::WeightedSigners, proof: axelar_soroban_interfaces::types::Proof) {

    }
}

impl AxelarGateway {
    fn contract_call_approval_key(
        command_id: Hash,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: Hash,
    ) -> DataKey {
        DataKey::ContractCallApproval(ContractCallApprovalKey {
            command_id,
            source_chain,
            source_address,
            contract_address,
            payload_hash,
        })
    }

    fn approve_contract_call(env: &Env, command_id: Hash, approval: types::ContractCallApproval) {
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
