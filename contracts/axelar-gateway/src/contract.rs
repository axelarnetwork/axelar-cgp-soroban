use crate::error::ContractError;
use crate::types::{CommandType, Message, Proof, WeightedSigners};
use axelar_soroban_std::ensure;
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Vec};

use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::{auth, event};

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Parameters for extending the contract instance and its instance storage.
///
/// If the instance's time to live falls below 14 days, it will be extended by 60 days.
///
/// If at least one message is approved every 14 days, the instance should never be archived.
const LEDGERS_PER_DAY: u32 = (24 * 3600) / 5;
const INSTANCE_TTL_THRESHOLD: u32 = 14 * LEDGERS_PER_DAY;
const INSTANCE_TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGateway {
    /// Initialize the gateway
    pub fn initialize(
        env: Env,
        owner: Address,
        operator: Address,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        previous_signers_retention: u64,
        initial_signers: Vec<WeightedSigners>,
    ) -> Result<(), ContractError> {
        ensure!(
            env.storage()
                .instance()
                .get::<DataKey, bool>(&DataKey::Initialized)
                .is_none(),
            ContractError::AlreadyInitialized
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

    /// Sends a message to the specified destination chain and contarct address with a given payload.
    ///
    /// This function is the entry point for general message passing between chains.
    ///
    /// A registered chain name on Axelar must be used for `destination_chain`.
    pub fn call_contract(
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

    /// Checks if a message is approved
    ///
    /// Determines whether a given message, identified by its `source_chain` and `message_id`, is approved.
    ///
    /// Returns true if a message with the given `payload_hash`  is approved.
    pub fn is_message_approved(
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

    /// Checks if a message is executed.
    ///
    /// Returns true if the message is executed, false otherwise.
    pub fn is_message_executed(env: Env, source_chain: String, message_id: String) -> bool {
        let message_approval = Self::message_approval(&env, source_chain, message_id);

        message_approval == MessageApprovalValue::Executed
    }

    /// Validates if a message is approved. If message was in approved status, status is updated to executed to avoid
    /// replay.
    ///
    /// `caller` must be the intended `destination_address` of the contract call for validation to succeed.
    pub fn validate_message(
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
            message_id,
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

    /// Approves a collection of messages.
    pub fn approve_messages(
        env: Env,
        messages: Vec<Message>,
        proof: Proof,
    ) -> Result<(), ContractError> {
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        auth::validate_proof(&env, &data_hash, proof)?;

        ensure!(!messages.is_empty(), ContractError::EmptyMessages);

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

        Self::extend_instance_ttl(&env);

        Ok(())
    }

    // TODO: add docstring about how bypass_rotation_delay supposed to be used.
    pub fn rotate_signers(
        env: Env,
        signers: WeightedSigners,
        proof: Proof,
        bypass_rotation_delay: bool,
    ) -> Result<(), ContractError> {
        if bypass_rotation_delay {
            Self::operator(&env)?.require_auth();
        }

        let data_hash: BytesN<32> = signers.signers_rotation_hash(&env);

        let is_latest_signers = auth::validate_proof(&env, &data_hash, proof)?;
        ensure!(
            bypass_rotation_delay || is_latest_signers,
            ContractError::NotLatestSigners
        );

        auth::rotate_signers(&env, &signers, !bypass_rotation_delay)?;

        Ok(())
    }

    pub fn transfer_operatorship(env: Env, new_operator: Address) -> Result<(), ContractError> {
        let operator: Address = Self::operator(&env)?;
        operator.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::Operator, &new_operator);

        event::transfer_operatorship(&env, operator, new_operator);

        Ok(())
    }

    pub fn operator(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Operator)
            .ok_or(ContractError::NotInitialized)
    }

    pub fn epoch(env: &Env) -> Result<u64, ContractError> {
        auth::epoch(env)
    }

    pub fn version(env: Env) -> String {
        String::from_str(&env, CONTRACT_VERSION)
    }

    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
        Self::owner(&env)?.require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Self::start_migrating(&env);

        Ok(())
    }

    pub fn migrate(env: Env, _migration_data: ()) -> Result<(), ContractError> {
        // DO NOT REMOVE THIS LINE, IT PREVENTS THE CONTRACT FROM BEING MIGRATED MULTIPLE TIMES
        Self::assert_is_migrating(&env)?;

        // Add migration logic here as needed


        // DO NOT REMOVE THIS LINE, IT PREVENTS THE CONTRACT FROM BEING MIGRATED MULTIPLE TIMES
        Self::complete_migration(&env);
        Ok(())
    }

    fn assert_is_migrating(env: &Env) -> Result<(), ContractError> {
        let is_migrating = env.storage().instance().get::<DataKey, bool>(&DataKey::Migrating).unwrap_or(false);

        if !is_migrating {
            Err(ContractError::MigrationAlreadyCompleted)
        } else {
            Ok(())
        }
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) -> Result<(), ContractError> {
        let owner: Address = Self::owner(&env)?;
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);

        Ok(())
    }

    pub fn owner(env: &Env) -> Result<Address, ContractError> {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .ok_or(ContractError::NotInitialized)
    }

    pub fn epoch_by_signers_hash(
        env: &Env,
        signers_hash: BytesN<32>,
    ) -> Result<u64, ContractError> {
        auth::epoch_by_signers_hash(env, signers_hash)
    }

    pub fn signers_hash_by_epoch(env: &Env, epoch: u64) -> Result<BytesN<32>, ContractError> {
        auth::signers_hash_by_epoch(env, epoch)
    }

    /// Get the message approval value by `source_chain` and `message_id`, defaulting to `MessageNotApproved`
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

    fn extend_instance_ttl(env: &Env) {
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    fn start_migrating(env: &Env) {
        env.storage().instance().set(&DataKey::Migrating, &true);
    }

    fn complete_migration(env: &Env) {
        env.storage().instance().set(&DataKey::Migrating, &false);
    }
}
