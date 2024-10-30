use soroban_sdk::{Address, Env, Symbol};

pub fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (
        Symbol::new(env, "ownership_transferred"),
        previous_owner,
        new_owner,
    );
    env.events().publish(topics, ());
}

pub fn add_operator(env: &Env, operator: Address) {
    let topics = (Symbol::new(env, "operator_added"), operator);
    env.events().publish(topics, ());
}

pub fn remove_operator(env: &Env, operator: Address) {
    let topics = (Symbol::new(env, "operator_removed"), operator);
    env.events().publish(topics, ());
}
