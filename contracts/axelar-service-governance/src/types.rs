use soroban_sdk::contracttype;

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
    pub command: U256,
    pub target: Address,
    pub func: Symbol,
    pub args: Vec<Val>,
    pub eta: U256,
}
