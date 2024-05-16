use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub enum DataKey {
    Initialized,
    AuthModule,
    Gateway,
    TimeLockProposal(Hash),
    MultisigProposal(Hash),
}
