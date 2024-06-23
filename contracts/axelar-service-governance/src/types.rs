use soroban_sdk::{contracttype, Address, Symbol, Val, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ServiceGovernanceCommandType {
    ScheduleTimeLockProposal,
    CancelTimeLockProposal,
    ApproveMultisigProposal,
    CancelMultisigApproval,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GovernanceProposal {
    pub command: u64,
    pub target: Address,
    pub func: Symbol,
    pub args: Vec<Val>,
    pub eta: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProposalKey {
    pub target: Address,
    pub func: Symbol,
    pub args: Vec<Val>,
}
