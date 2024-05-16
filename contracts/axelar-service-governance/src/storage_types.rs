use soroban_sdk::{contracttype, Address};

use axelar_soroban_std::types::Hash;

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    AuthModule,
    Gateway,
    TimeLockProposal(Hash),
    MultisigProposal(Hash),
}
