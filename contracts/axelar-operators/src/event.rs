use soroban_sdk::{symbol_short, Address, Env};

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (symbol_short!("ownership"), previous_owner, new_owner);
    env.events().publish(topics, ());
}

pub(crate) fn add_operator(env: &Env, operator: Address) {
    let topics = (symbol_short!("added"), operator);
    env.events().publish(topics, ());
}

pub(crate) fn remove_operator(env: &Env, operator: Address) {
    let topics = (symbol_short!("removed"), operator);
    env.events().publish(topics, ());
}
