use soroban_sdk::{Address, Env, Symbol};

pub fn set_admin(env: &Env, previous_admin: Address, new_admin: Address) {
    let topics = (Symbol::new(env, "set_admin"), previous_admin, new_admin);
    env.events().publish(topics, ());
}

pub fn add_minter(env: &Env, minter: Address) {
    let topics = (Symbol::new(env, "add_minter"), minter);
    env.events().publish(topics, ());
}
