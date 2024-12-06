use soroban_sdk::{Bytes, Env, String, Symbol};

pub fn set_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (
        Symbol::new(env, "trusted_address_set"),
        chain,
        trusted_address,
    );
    env.events().publish(topics, ());
}

pub fn remove_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (
        Symbol::new(env, "trusted_address_removed"),
        chain,
        trusted_address,
    );
    env.events().publish(topics, ());
}

pub fn executed(
    env: &Env,
    source_chain: String,
    message_id: String,
    source_address: String,
    payload: Bytes,
) {
    let topics = (
        Symbol::new(env, "executed"),
        source_chain,
        message_id,
        source_address,
    );
    env.events().publish(topics, (payload,));
}
