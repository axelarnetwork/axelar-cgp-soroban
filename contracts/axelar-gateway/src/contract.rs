use axelar_soroban_interfaces::types::{Message, Proof};
use axelar_soroban_std::ensure;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::types::CommandType;
use crate::{auth, event};
use axelar_soroban_interfaces::axelar_gateway::GatewayAuthError;
use axelar_soroban_interfaces::{axelar_gateway::AxelarGatewayInterface, types::WeightedSigners};

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn initialize(
        env: Env,
        operator: Address,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        previous_signers_retention: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), GatewayAuthError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            GatewayAuthError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Operator, &operator);

        auth::initialize_auth(
            env,
            domain_separator,
            minimum_rotation_delay,
            previous_signers_retention,
            initial_signers,
        )?;

        Ok(())
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
    ) -> Result<(), GatewayAuthError> {
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        auth::validate_proof(&env, &data_hash, proof.clone())?;

        ensure!(!messages.is_empty(), GatewayAuthError::EmptyMessages);

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

        Ok(())
    }

    // TODO: add docstring about how bypass_rotation_delay supposed to be used.
    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        bypass_rotation_delay: bool,
    ) -> Result<(), GatewayAuthError> {
        if bypass_rotation_delay {
            Self::operator(&env)?.require_auth();
        }

        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::RotateSigners, signers.clone()).to_xdr(&env))
            .into();

        ensure!(
            !env.storage()
                .persistent()
                .has(&DataKey::RotationExecuted(data_hash.clone())),
            GatewayAuthError::RotationAlreadyExecuted
        );

        let is_latest_signers = auth::validate_proof(&env, &data_hash, proof)?;
        ensure!(
            bypass_rotation_delay || is_latest_signers,
            GatewayAuthError::NotLatestSigners
        );

        env.storage()
            .persistent()
            .set(&DataKey::RotationExecuted(data_hash), &true);

        auth::rotate_signers(&env, &signers, !bypass_rotation_delay)
    }

    fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), GatewayAuthError> {
        let operator: Address = Self::operator(&env)?;
        operator.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Operator, &new_operator);

        event::transfer_operatorship(&env, operator, new_operator);
        Ok(())
    }

    fn operator(env: &Env) -> Result<Address, GatewayAuthError> {
        env.storage()
            .instance()
            .get(&DataKey::Operator)
            .ok_or(GatewayAuthError::NotInitialized)
    }
}

impl AxelarGateway {
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

    fn message_approval_hash(env: &Env, message: Message) -> MessageApprovalValue {
        MessageApprovalValue::Approved(env.crypto().keccak256(&message.to_xdr(env)).into())
    }
}
