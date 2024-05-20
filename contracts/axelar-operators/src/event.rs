use soroban_sdk::{symbol_short, Address, Env, Val, Vec};

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (symbol_short!("ownership"), symbol_short!("transfer"));
    env.events().publish(topics, (previous_owner, new_owner));
}

pub(crate) fn add_operator(env: &Env, operator: Address) {
    let topics = (symbol_short!("operator"), symbol_short!("added"));
    env.events().publish(topics, (operator,));
}

pub(crate) fn remove_operator(env: &Env, operator: Address) {
    let topics = (symbol_short!("operator"), symbol_short!("removed"));
    env.events().publish(topics, (operator,));
}

pub(crate) fn execute(env: &Env, target: Address, func: Address, args: Vec<Val>) {
    let topics = (symbol_short!("executed"),);
    env.events().publish(topics, (target, func, args));
}
