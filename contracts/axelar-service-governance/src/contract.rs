use soroban_sdk::{
    contract, contractimpl, panic_with_error, Address, Bytes, Env, String, Symbol, Val, Vec,
};

use axelar_soroban_std::types::Hash;

use axelar_soroban_interfaces::axelar_auth_verifier::AxelarAuthVerifierClient;
use axelar_soroban_interfaces::axelar_gateway::AxelarGatewayClient;
use axelar_soroban_interfaces::types::Proof;

use crate::storage_types::DataKey;
use crate::types::{GovernanceProposal, ServiceGovernanceCommandType};
use crate::{error::Error, event};
use axelar_soroban_interfaces::axelar_service_governance::AxelarServiceGovernanceInterface;

#[contract]
pub struct AxelarServiceGovernance;

#[contractimpl]
impl AxelarServiceGovernanceInterface for AxelarServiceGovernance {
    fn initialize(env: Env, auth_module: Address, gateway: Address) {
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

        env.storage().instance().set(&DataKey::Gateway, &gateway);
    }

    fn execute(
        env: Env,
        command_id: Hash,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));

        // Validate the contract call was approved by the gateway
        if !gateway.validate_contract_call(
            &env.current_contract_address(),
            &command_id,
            &source_chain,
            &source_address,
            &env.crypto().keccak256(&payload),
        ) {
            panic_with_error!(env, Error::NotApproved);
        };

        // TODO read from payload
        let proposal = GovernanceProposal {
            command: command_id,
            target: env.current_contract_address(),
            func: Symbol::new(env, "execute"),
            args: (),
            eta: env.block().timestamp() + 60 * 60 * 24,
        };

        let proposal_hash = env
            .crypto()
            .keccak256(&(proposal.target, proposal.func, proposal.args.clone()).to_xdr());

        if (proposal.command == ServiceGovernanceCommandType::ScheduleTimeLockProposal) {
            env.storage()
                .instance()
                .set(&DataKey::TimeLockProposal(proposal_hash), &true);
        } else if (proposal.command == ServiceGovernanceCommandType::CancelTimeLockProposal) {
            env.storage()
                .instance()
                .remove(&DataKey::TimeLockProposal(proposal_hash));
        } else if (proposal.command == ServiceGovernanceCommandType::ApproveMultisigProposal) {
            env.storage()
                .instance()
                .set(&DataKey::MultisigProposal(proposal_hash), &true);
        } else if (proposal.command == ServiceGovernanceCommandType::CancelMultisigApproval) {
            env.storage()
                .instance()
                .remove(&DataKey::MultisigProposal(proposal_hash));
        } else {
            panic_with_error!(env, Error::InvalidCommand);
        }
    }

    fn execute_proposal(env: Env, target: Address, func: Symbol, args: Vec<Val>) -> Val {
        let proposal_hash = env
            .crypto()
            .keccak256(&(target, func, args.clone()).to_xdr());

        let proposal = match env
            .storage()
            .instance()
            .get(&DataKey::TimeLockProposal(proposal_hash))
        {
            Some(proposal) => proposal,
            None => panic_with_error!(env, Error::ProposalNotFound),
        };

        if proposal.eta > env.block().timestamp() {
            panic_with_error!(env, Error::ProposalNotReady);
        }

        env.storage()
            .instance()
            .remove(&DataKey::TimeLockProposal(proposal_hash));

        let res: Val = env.invoke_contract(&target, &func, args);

        res
    }

    fn execute_multisig_proposal(
        env: Env,
        target: Address,
        func: Symbol,
        args: Vec<Val>,
        proof: Proof,
    ) -> Val {
        let proposal_hash = env
            .crypto()
            .keccak256(&(target, func, args.clone()).to_xdr());

        let proposal = match env
            .storage()
            .instance()
            .get(&DataKey::MultisigProposal(proposal_hash))
        {
            Some(proposal) => proposal,
            None => panic_with_error!(env, Error::ProposalNotFound),
        };

        let auth_module = Self::auth_module(&env);

        auth_module.validate_proof(&proposal_hash, &proof);

        env.storage()
            .instance()
            .remove(&DataKey::MultisigProposal(proposal_hash));

        let res: Val = env.invoke_contract(&target, &func, args);

        res
    }
}

#[contractimpl]
impl AxelarServiceGovernance {
    fn gateway(env: &Env) -> Address {
        env.storage().instance().get(&"gateway").unwrap()
    }

    fn auth_module(env: &Env) -> AxelarAuthVerifierClient {
        AxelarAuthVerifierClient::new(
            env,
            match &env.storage().instance().get(&DataKey::AuthModule) {
                Some(auth) => auth,
                None => panic_with_error!(env, Error::Uninitialized),
            },
        )
    }
}
