use soroban_sdk::{symbol_short, String, Address, Env};

pub(crate) fn set_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (symbol_short!("set"), chain, trusted_address);
    env.events().publish(topics, ());
}

pub(crate) fn remove_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (symbol_short!("removed"), chain, trusted_address);
    env.events().publish(topics, ());
}

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (symbol_short!("ownership"), previous_owner, new_owner);
    env.events().publish(topics, ());
}