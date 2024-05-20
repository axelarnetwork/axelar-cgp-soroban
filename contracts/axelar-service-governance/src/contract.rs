use soroban_sdk::xdr::{FromXdr, ToXdr};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Bytes, Env, String, Symbol, Val, Vec,
};

use axelar_soroban_interfaces::axelar_gateway::AxelarGatewayClient;

use crate::storage_types::DataKey;
use crate::types::{GovernanceProposal, ProposalKey, ServiceGovernanceCommandType};
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_service_governance::AxelarServiceGovernanceInterface;

#[contract]
pub struct AxelarServiceGovernance;

#[contractimpl]
impl AxelarServiceGovernanceInterface for AxelarServiceGovernance {
    fn initialize(env: Env, multisig: Address, gateway: Address, minumum_time_delay: u64) {
        if env
            .storage()
            .instance()
            .get(&DataKey::Initialized)
            .unwrap_or(false)
        {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);

        env.storage().instance().set(&DataKey::Multisig, &multisig);

        env.storage().instance().set(&DataKey::Gateway, &gateway);

        env.storage()
            .instance()
            .set(&DataKey::MinimumTimeDelay, &minumum_time_delay);
    }

    fn execute(
        env: Env,
        message_id: String,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));

        // Validate the contract call was approved by the gateway
        if !gateway.validate_message(
            &env.current_contract_address(),
            &message_id,
            &source_chain,
            &source_address,
            &env.crypto().keccak256(&payload),
        ) {
            panic_with_error!(env, Error::NotApproved);
        };

        let proposal = match GovernanceProposal::from_xdr(&env, &payload) {
            Ok(x) => x,
            Err(_) => panic_with_error!(env, Error::InvalidPayload),
        };

        let proposal_hash = env.crypto().keccak256(
            &ProposalKey {
                target: proposal.target.clone(),
                func: proposal.func.clone(),
                args: proposal.args.clone(),
            }
            .to_xdr(&env),
        );

        if proposal.command == ServiceGovernanceCommandType::ScheduleTimeLockProposal as u64 {
            if proposal.eta < env.ledger().timestamp() + Self::minimum_time_delay(&env) {
                panic_with_error!(env, Error::InvalidEta);
            }

            env.storage().instance().set(
                &DataKey::TimeLockProposal(proposal_hash.clone()),
                &proposal.eta,
            );

            event::schedule_timelock_proposal(
                &env,
                proposal.target,
                proposal.func,
                proposal.args,
                proposal.eta,
            );
        } else if proposal.command == ServiceGovernanceCommandType::CancelTimeLockProposal as u64 {
            env.storage()
                .instance()
                .remove(&DataKey::TimeLockProposal(proposal_hash.clone()));

            event::cancel_timelock_proposal(&env, proposal.target, proposal.func, proposal.args);
        } else if proposal.command == ServiceGovernanceCommandType::ApproveMultisigProposal as u64 {
            env.storage()
                .instance()
                .set(&DataKey::MultisigProposal(proposal_hash.clone()), &true);

            event::schedule_multisig_proposal(&env, proposal.target, proposal.func, proposal.args);
        } else if proposal.command == ServiceGovernanceCommandType::CancelMultisigApproval as u64 {
            env.storage()
                .instance()
                .remove(&DataKey::MultisigProposal(proposal_hash.clone()));

            event::cancel_multisig_proposal(&env, proposal.target, proposal.func, proposal.args);
        } else {
            panic_with_error!(env, Error::InvalidCommand);
        }
    }

    fn get_proposal_eta(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> u64 {
        let proposal_hash = env
            .crypto()
            .keccak256(&ProposalKey { target, func, args }.to_xdr(&env));

        match env
            .storage()
            .instance()
            .get(&DataKey::TimeLockProposal(proposal_hash))
        {
            Some(eta) => eta,
            None => panic_with_error!(env, Error::ProposalNotFound),
        }
    }

    fn execute_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val {
        let proposal_hash = env.crypto().keccak256(
            &ProposalKey {
                target: target.clone(),
                func: func.clone(),
                args: args.clone(),
            }
            .to_xdr(&env),
        );

        let eta: u64 = match env
            .storage()
            .instance()
            .get(&DataKey::TimeLockProposal(proposal_hash.clone()))
        {
            Some(proposal) => proposal,
            None => panic_with_error!(env, Error::ProposalNotFound),
        };

        if eta > env.ledger().timestamp() {
            panic_with_error!(env, Error::ProposalNotReady);
        }

        env.storage()
            .instance()
            .remove(&DataKey::TimeLockProposal(proposal_hash.clone()));

        let res: Val = env.invoke_contract(&target, &func, args.clone());

        event::execute_timelock_proposal(&env, target, func, args);

        res
    }

    fn is_multisig_proposal_approved(
        env: Env,
        target: Address,
        func: Symbol,
        args: Vec<Val>,
    ) -> bool {
        let proposal_hash = env
            .crypto()
            .keccak256(&ProposalKey { target, func, args }.to_xdr(&env));

        env.storage()
            .instance()
            .get(&DataKey::MultisigProposal(proposal_hash.clone()))
            .unwrap()
    }

    fn execute_multisig_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val {
        Self::multisig(&env).require_auth();

        let proposal_hash = env.crypto().keccak256(
            &ProposalKey {
                target: target.clone(),
                func: func.clone(),
                args: args.clone(),
            }
            .to_xdr(&env),
        );

        let is_approved = env
            .storage()
            .instance()
            .get(&DataKey::MultisigProposal(proposal_hash.clone()))
            .unwrap_or(false);

        if !is_approved {
            panic_with_error!(env, Error::ProposalNotFound);
        }

        env.storage()
            .instance()
            .remove(&DataKey::MultisigProposal(proposal_hash.clone()));

        let res: Val = env.invoke_contract(&target, &func, args.clone());

        event::execute_multisig_proposal(&env, target, func, args);

        res
    }
}

#[contractimpl]
impl AxelarServiceGovernance {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Gateway).unwrap()
    }

    fn multisig(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Multisig).unwrap()
    }

    fn minimum_time_delay(env: &Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::MinimumTimeDelay)
            .unwrap()
    }
}
