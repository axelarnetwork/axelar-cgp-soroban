use soroban_sdk::{symbol_short, Address, Env};

use crate::types::WeightedSigners;
use axelar_soroban_std::types::Hash;

pub(crate) fn transfer_operatorship(env: &Env, signer_set: WeightedSigners, signer_set_hash: Hash) {
    let topics = (symbol_short!("transfer"), signer_set_hash);
    env.events().publish(topics, (signer_set,)); // TODO: use a tuple or the type directly?
}

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (symbol_short!("ownership"), previous_owner, new_owner);
    env.events().publish(topics, ());
}
