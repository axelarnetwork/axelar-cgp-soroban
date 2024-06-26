use axelar_soroban_interfaces::types::Message;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Bytes, BytesN, Env, String};

use axelar_soroban_interfaces::axelar_auth_verifier::AxelarAuthVerifierClient;

use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::types::CommandType;
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_gateway::AxelarGatewayInterface;

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn initialize(env: Env, auth_module: Address, operator: Address) {
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

        env.storage().instance().set(&DataKey::Operator, &operator);
    }

    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    ) {
        caller.require_auth();

        let payload_hash = env.crypto().keccak256(&payload).into();

        event::call_contract(
            &env,
            caller,
            destination_chain,
            destination_address,
            payload,
            payload_hash,
        );
    }

    fn is_message_approved(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool {
        let message_approval =
            Self::message_approval(&env, message_id.clone(), source_chain.clone());

        message_approval
            == Self::message_approval_hash(
                &env,
                Message {
                    message_id,
                    source_chain,
                    source_address,
                    contract_address,
                    payload_hash,
                },
            )
    }

    fn is_message_executed(env: Env, message_id: String, source_chain: String) -> bool {
        let message_approval = Self::message_approval(&env, message_id, source_chain);

        message_approval == MessageApprovalValue::Executed
    }

    fn validate_message(
        env: Env,
        caller: Address,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool {
        caller.require_auth();

        let key = MessageApprovalKey {
            message_id: message_id.clone(),
            source_chain: source_chain.clone(),
        };
        let message_approval = Self::message_approval_by_key(&env, key.clone());
        let message = Message {
            message_id: message_id.clone(),
            source_chain,
            source_address,
            contract_address: caller,
            payload_hash,
        };

        if message_approval == Self::message_approval_hash(&env, message) {
            env.storage().persistent().set(
                &DataKey::MessageApproval(key),
                &MessageApprovalValue::Executed,
            );

            event::execute_contract_call(&env, message_id);

            return true;
        }

        false
    }

    fn approve_messages(
        env: Env,
        messages: soroban_sdk::Vec<axelar_soroban_interfaces::types::Message>,
        proof: axelar_soroban_interfaces::types::Proof,
    ) {
        let data_hash = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        let auth_module = Self::auth_module(&env);

        auth_module.validate_proof(&data_hash, &proof);

        if messages.is_empty() {
            panic_with_error!(env, Error::EmptyMessages);
        }

        for message in messages.into_iter() {
            let key = MessageApprovalKey {
                message_id: message.message_id.clone(),
                source_chain: message.source_chain.clone(),
            };

            // Prevent replay if message is already approved/executed
            let message_approval = Self::message_approval_by_key(&env, key.clone());
            if message_approval != MessageApprovalValue::NotApproved {
                continue;
            }

            env.storage().persistent().set(
                &DataKey::MessageApproval(key),
                &Self::message_approval_hash(&env, message.clone()),
            );

            event::approve_message(&env, message);
        }
    }

    fn rotate_signers(
        env: Env,
        signers: axelar_soroban_interfaces::types::WeightedSigners,
        proof: axelar_soroban_interfaces::types::Proof,
    ) {
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::RotateSigners, signers.clone()).to_xdr(&env))
            .into();

        let auth_module = Self::auth_module(&env);

        // TODO: Add rotation delay governance
        if env
            .storage()
            .persistent()
            .has(&DataKey::RotationExecuted(data_hash.clone()))
        {
            panic_with_error!(env, Error::RotationAlreadyExecuted);
        }

        let is_latest_signers = auth_module.validate_proof(&data_hash, &proof);
        if !is_latest_signers {
            panic_with_error!(env, Error::NotLatestSigners);
        }

        env.storage()
            .persistent()
            .set(&DataKey::RotationExecuted(data_hash), &true);

        auth_module.rotate_signers(&signers, &true);

        event::rotate_signers(&env, signers);
    }
}

impl AxelarGateway {
    fn auth_module(env: &Env) -> AxelarAuthVerifierClient {
        AxelarAuthVerifierClient::new(
            env,
            match &env.storage().instance().get(&DataKey::AuthModule) {
                Some(auth) => auth,
                None => panic_with_error!(env, Error::Uninitialized),
            },
        )
    }

    fn message_approval_hash(env: &Env, message: Message) -> MessageApprovalValue {
        MessageApprovalValue::Approved(env.crypto().keccak256(&message.to_xdr(env)).into())
    }

    /// Get the message approval value by message_id and source_chain, defaulting to `MessageNotApproved`
    fn message_approval(
        env: &Env,
        message_id: String,
        source_chain: String,
    ) -> MessageApprovalValue {
        let key = MessageApprovalKey {
            message_id,
            source_chain,
        };

        Self::message_approval_by_key(env, key)
    }

    /// Get the message approval value by key, defaulting to `MessageNotApproved`
    fn message_approval_by_key(env: &Env, key: MessageApprovalKey) -> MessageApprovalValue {
        env.storage()
            .persistent()
            .get(&DataKey::MessageApproval(key))
            .unwrap_or(MessageApprovalValue::NotApproved)
    }
}
