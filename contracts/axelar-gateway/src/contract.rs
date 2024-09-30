use core::panic;
use axelar_soroban_interfaces::types::{Message, ProofSignature, ProofSigner, WeightedSigner};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{contract, contractimpl, crypto::Hash, panic_with_error, Address, Bytes, BytesN, Env, Vec, String};

use crate::storage_types::{DataKey, MessageApprovalKey, MessageApprovalValue};
use crate::types::CommandType;
use crate::{error::Error, event};
use axelar_soroban_interfaces::{
    axelar_gateway::AxelarGatewayInterface,
    types::{Proof, WeightedSigners},
};

#[contract]
pub struct AxelarGateway;

#[contractimpl]
impl AxelarGatewayInterface for AxelarGateway {
    fn initialize(
        env: Env,
        operator: Address,
        domain_separator: BytesN<32>,
        minimum_rotation_delay: u64,
        previous_signer_retention: u64,
        initial_signers: Vec<WeightedSigners>
    ) {
        if env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
        {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Operator, &operator);

        env.storage().instance().set(&DataKey::Epoch, &0_u64);

        // TODO: Do we need to manually expose these in a query, or can it be read directly off of storage in Stellar?
        env.storage().instance().set(
            &DataKey::PreviousSignerRetention,
            &previous_signer_retention,
        );

        env.storage()
            .instance()
            .set(&DataKey::DomainSeparator, &domain_separator);

        env.storage()
            .instance()
            .set(&DataKey::MinimumRotationDelay, &minimum_rotation_delay);

        if initial_signers.is_empty() {
            panic_with_error!(env, Error::InvalidOperators);
        }

        for signers in initial_signers.into_iter() {
            Self::rotate_signers_set(&env, signers.clone(), false);
        }
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
        let data_hash: BytesN<32> = env
            .crypto()
            .keccak256(&(CommandType::ApproveMessages, messages.clone()).to_xdr(&env))
            .into();

        Self::validate_proof(&env, data_hash.clone(), proof.clone());

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

        // TODO: Add rotation delay governance
        if env
            .storage()
            .persistent()
            .has(&DataKey::RotationExecuted(data_hash.clone())) {
            panic_with_error!(env, Error::RotationAlreadyExecuted);
        }

        let is_latest_signers = Self::validate_proof(&env, data_hash.clone(), proof.clone());
        if !is_latest_signers {
            panic_with_error!(env, Error::NotLatestSigners);
        }

        env.storage()
            .persistent()
            .set(&DataKey::RotationExecuted(data_hash), &true);

        Self::rotate_signers_set(&env, signers, true);
    }

    fn transfer_operatorship(env: Env, new_operator: Address) {
        let operator: Address = Self::operator(&env);
        operator.require_auth();

        env.storage().instance().set(&DataKey::Operator, &new_operator);

        event::transfer_operatorship(&env, operator, new_operator);
    }

    fn operator(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Operator).unwrap()
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

    fn validate_proof(env: &Env, data_hash: BytesN<32>, proof: Proof) -> bool {
        let signer_set = proof.weighted_signers();

        let signer_hash: BytesN<32> = env.crypto().keccak256(&signer_set.to_xdr(&env)).into();

        let signer_epoch: u64 = env
            .storage()
            .persistent()
            .get(&DataKey::EpochBySignerHash(signer_hash.clone()))
            .unwrap_or(0);

        if signer_epoch == 0 {
            panic_with_error!(env, Error::InvalidSigners);
        }

        let current_epoch: u64 = Self::epoch(&env);

        let is_latest_signers: bool = signer_epoch == current_epoch;

        let previous_signer_retention: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PreviousSignerRetention)
            .unwrap();

        if current_epoch - signer_epoch > previous_signer_retention {
            panic_with_error!(env, Error::InvalidSigners);
        }

        let msg_hash = Self::message_hash_to_sign(&env, signer_hash, data_hash);

        if !Self::validate_signatures(&env, msg_hash, proof.clone()) {
            panic_with_error!(env, Error::InvalidSignatures);
        }

        is_latest_signers
    }

    fn message_hash_to_sign(env: &Env, signer_hash: BytesN<32>, data_hash: BytesN<32>) -> Hash<32> {
        let domain_separator: BytesN<32> = env
            .storage()
            .instance()
            .get(&DataKey::DomainSeparator)
            .unwrap();

        let mut msg: Bytes = domain_separator.into();
        msg.extend_from_array(&signer_hash.to_array());
        msg.extend_from_array(&data_hash.to_array());

        // TODO: use an appropriate non tx overlapping prefix
        env.crypto().keccak256(&msg)
    }

    fn rotate_signers_set(env: &Env, new_signers: WeightedSigners, enforce_rotation_delay: bool) {
        let operator: Address = Self::operator(&env);
        operator.require_auth();

        Self::validate_signers(env, &new_signers);

        Self::update_rotation_timestamp(env, enforce_rotation_delay);

        let new_signer_hash: BytesN<32> = env
            .crypto()
            .keccak256(&new_signers.clone().to_xdr(env))
            .into();
        let new_epoch: u64 = Self::epoch(&env) + 1;

        env.storage().instance().set(&DataKey::Epoch, &new_epoch);

        env.storage()
            .persistent()
            .set(&DataKey::SignerHashByEpoch(new_epoch), &new_signer_hash);

        // If new_signers has been rotated to before, we will overwrite the epoch to point to the latest
        env.storage().persistent().set(
            &DataKey::EpochBySignerHash(new_signer_hash.clone()),
            &new_epoch,
        );

        event::rotate_signers_set(env, new_epoch, new_signers, new_signer_hash);
    }

    fn update_rotation_timestamp(env: &Env, enforce_rotation_delay: bool) {
        let minimum_rotation_delay: u64 = env
            .storage()
            .instance()
            .get(&DataKey::MinimumRotationDelay)
            .unwrap();

        let last_rotation_timestamp: u64 = env
            .storage()
            .instance()
            .get(&DataKey::LastRotationTimestamp)
            .unwrap_or(0);

        let current_timestamp = env.ledger().timestamp();

        if enforce_rotation_delay
            && (current_timestamp - last_rotation_timestamp < minimum_rotation_delay)
        {
            panic_with_error!(env, Error::InsufficientRotationDelay);
        }

        env.storage()
            .instance()
            .set(&DataKey::LastRotationTimestamp, &current_timestamp);
    }

    fn validate_signatures(env: &Env, msg_hash: Hash<32>, proof: Proof) -> bool {
        let mut total_weight = 0u128;

        for ProofSigner {
            signer:
                WeightedSigner {
                    signer: public_key,
                    weight,
                },
            signature,
        } in proof.signers.iter()
        {
            if let ProofSignature::Signed(signature) = signature {
                env.crypto()
                    .ed25519_verify(&public_key, msg_hash.to_bytes().as_ref(), &signature);

                total_weight = total_weight.checked_add(weight).unwrap();

                if total_weight >= proof.threshold {
                    return true;
                }
            }
        }

        false
    }

    /// Check if signer set is valid, i.e signer/pub key hash are in sorted order,
    /// weights are non-zero and sum to at least threshold
    fn validate_signers(env: &Env, weighted_signers: &WeightedSigners) {
        if weighted_signers.signers.is_empty() {
            panic_with_error!(env, Error::InvalidSigners);
        }

        // TODO: what's the min address/hash?
        let mut previous_signer = BytesN::<32>::from_array(env, &[0; 32]);
        let mut total_weight = 0u128;

        for signer in weighted_signers.signers.iter() {
            if previous_signer >= signer.signer {
                panic_with_error!(env, Error::InvalidSigners);
            }

            if signer.weight == 0 {
                panic_with_error!(env, Error::InvalidWeights);
            }

            previous_signer = signer.signer;
            total_weight = total_weight.checked_add(signer.weight).unwrap();
        }

        let threshold = weighted_signers.threshold;
        if threshold == 0 || total_weight < threshold {
            panic_with_error!(env, Error::InvalidThreshold);
        }
    }

    fn epoch(env: &Env) -> u64 {
        env.storage().instance().get(&DataKey::Epoch).unwrap()
    }

}
