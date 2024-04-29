use axelar_soroban_interfaces::types::WeightedSigners;
use soroban_sdk::{symbol_short, Address, Env};

use axelar_soroban_std::types::Hash;

pub(crate) fn rotate_signers(env: &Env, epoch: u64, signers: WeightedSigners, signer_hash: Hash) {
    let topics = (symbol_short!("rotated"), epoch, signer_hash);
    env.events().publish(topics, (signers,)); // TODO: use a tuple or the type directly?
}

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (symbol_short!("ownership"), previous_owner, new_owner);
    env.events().publish(topics, ());
}
