use crate::types::{CommandType, GatewayError, Message, Proof, WeightedSigners};
use axelar_soroban_std::ensure;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::{auth, event};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Interface for the Axelar Gateway.
// #[contractclient(name = "AxelarGatewayClient")]
pub trait AxelarGatewayInterface {
    /// Initialize the gateway
    fn initialize(
        env: Env,
        owner: Address,
        operator: Address,
        domain_separator: BytesN<32>,
        previous_signers_retention: u64,
        minimum_rotation_delay: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), GatewayError>;

    /// Call a contract on another chain with the given payload. The destination address can validate the contract call on the destination gateway.
    fn call_contract(
        env: Env,
        caller: Address,
        destination_chain: String,
        destination_address: String,
        payload: Bytes,
    );

    fn approve_messages(env: Env, messages: Vec<Message>, proof: Proof)
        -> Result<(), GatewayError>;

    /// Validate if a contract call with the given payload BytesN<32> and source caller info is approved,
    /// preventing re-validation (i.e distinct contract calls can be validated at most once).
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info is approved.
    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool;

    /// Return true if a contract call with the given payload BytesN<32> and source caller info has been executed.
    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool;

    fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        enforce_rotation_delay: bool,
    ) -> Result<(), GatewayError>;

    fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), GatewayError>;

    fn operator(env: &Env) -> Result<Address, GatewayError>;

    fn epoch(env: &Env) -> Result<u64, GatewayError>;

    fn version(env: Env) -> String;

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), GatewayError>;

    fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), GatewayError>;

    fn owner(env: &Env) -> Result<Address, GatewayError>;
}

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    /// Initialize the gateway
    fn initialize(
        env: Env,
        owner: Address,
        operator: Address,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        previous_signers_retention: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), GatewayError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            GatewayError::AlreadyInitialized
        );

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Owner, &owner);
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

    /// Call a contract on another chain with the given payload. The destination address can validate the contract call on the destination gateway.
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

    /// Return true if a contract call with the given payload BytesN<32> and source caller info is approved.
    fn is_message_approved(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        contract_address: Address,
        payload_hash: BytesN<32>,
    ) -> bool {
        let message_approval =
            Self::message_approval(&env, source_chain.clone(), message_id.clone());

        message_approval
            == Self::message_approval_hash(
                &env,
                Message {
                    source_chain,
                    message_id,
                    source_address,
                    contract_address,
                    payload_hash,
                },
            )
    }

    /// Return true if a contract call with the given payload BytesN<32> and source caller info has been executed.
    fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool {
        let message_approval = Self::message_approval(&env, source_chain, message_id);

        message_approval == MessageApprovalValue::Executed
    }

    /// Validate if a contract call with the given payload BytesN<32> and source caller info is approved,
    /// preventing re-validation (i.e distinct contract calls can be validated at most once).
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    fn validate_message(
        env: Env,
        caller: Address,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload_hash: BytesN<32>,
    ) -> bool {
        caller.require_auth();

        let key = MessageApprovalKey {
            source_chain: source_chain.clone(),
            message_id: message_id.clone(),
        };
        let message_approval = Self::message_approval_by_key(&env, key.clone());
        let message = Message {
            source_chain,
            message_id: message_id.clone(),
            source_address,
            contract_address: caller,
            payload_hash,
        };

        if message_approval == Self::message_approval_hash(&env, message.clone()) {
            env.storage().persistent().set(
                &DataKey::MessageApproval(key),
                &MessageApprovalValue::Executed,
            );

            event::execute_message(&env, message);

            return true;
        }

        false
    }

    fn approve_messages(
        env: Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), GatewayError> {
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        auth::validate_proof(&env, &data_hash, proof.clone())?;

        ensure!(!messages.is_empty(), GatewayError::EmptyMessages);

        for message in messages.into_iter() {
            let key = MessageApprovalKey {
                source_chain: message.source_chain.clone(),
                message_id: message.message_id.clone(),
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
    ) -> Result<(), GatewayError> {
        if bypass_rotation_delay {
            Self::operator(&env)?.require_auth();
        }

        let data_hash: BytesN<32> = signers.signers_rotation_hash(&env);

        let is_latest_signers = auth::validate_proof(&env, &data_hash, proof)?;
        ensure!(
            bypass_rotation_delay || is_latest_signers,
            GatewayError::NotLatestSigners
        );

        auth::rotate_signers(&env, &signers, !bypass_rotation_delay)?;

        Ok(())
    }

    fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), GatewayError> {
        let operator: Address = Self::operator(&env)?;
        operator.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Operator, &new_operator);

        event::transfer_operatorship(&env, operator, new_operator);

        Ok(())
    }

    fn operator(env: &Env) -> Result<Address, GatewayError> {
        env.storage()
            .instance()
            .get(&DataKey::Operator)
            .ok_or(GatewayError::NotInitialized)
    }

    fn epoch(env: &Env) -> Result<u64, GatewayError> {
        auth::epoch(env)
    }

    fn version(env: Env) -> String {
        String::from_str(&env, CONTRACT_VERSION)
    }

    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), GatewayError> {
        Self::owner(&env)?.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), GatewayError> {
        let owner: Address = Self::owner(&env)?;
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    fn owner(env: &Env) -> Result<Address, GatewayError> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(GatewayError::NotInitialized)
    }
}

impl AxelarGateway {
    /// Get the message approval value by source_chain and message_id, defaulting to `MessageNotApproved`
    fn message_approval(
        env: &Env,
        source_chain: String,
        message_id: String,
    ) -> MessageApprovalValue {
        let key = MessageApprovalKey {
            source_chain,
            message_id,
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
