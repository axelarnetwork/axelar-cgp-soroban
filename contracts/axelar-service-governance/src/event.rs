use soroban_sdk::{symbol_short, Address, Env, Symbol, Val, Vec};

pub(crate) fn schedule_timelock_proposal(
    env: &Env,
    target: Address,
    func: Symbol,
    args: Vec<Val>,
    eta: u64,
) {
    let topics = (symbol_short!("timelock"), symbol_short!("added"));
    env.events().publish(topics, (target, func, args, eta));
}
pub(crate) fn cancel_timelock_proposal(env: &Env, target: Address, func: Symbol, args: Vec<Val>) {
    let topics = (symbol_short!("timelock"), symbol_short!("canceled"));
    env.events().publish(topics, (target, func, args));
}
pub(crate) fn execute_timelock_proposal(env: &Env, target: Address, func: Symbol, args: Vec<Val>) {
    let topics = (symbol_short!("timelock"), symbol_short!("executed"));
    env.events().publish(topics, (target, func, args));
}

pub(crate) fn schedule_multisig_proposal(env: &Env, target: Address, func: Symbol, args: Vec<Val>) {
    let topics = (symbol_short!("multisig"), symbol_short!("added"));
    env.events().publish(topics, (target, func, args));
}
pub(crate) fn cancel_multisig_proposal(env: &Env, target: Address, func: Symbol, args: Vec<Val>) {
    let topics = (symbol_short!("multisig"), symbol_short!("canceled"));
    env.events().publish(topics, (target, func, args));
}
pub(crate) fn execute_multisig_proposal(env: &Env, target: Address, func: Symbol, args: Vec<Val>) {
    let topics = (symbol_short!("multisig"), symbol_short!("executed"));
    env.events().publish(topics, (target, func, args));
}
