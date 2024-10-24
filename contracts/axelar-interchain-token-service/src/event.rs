use soroban_sdk::{Address, Env, String, Symbol};

pub(crate) fn set_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (
        Symbol::new(env, "trusted_address_set"),
        chain,
        trusted_address,
    );
    env.events().publish(topics, ());
}

pub(crate) fn remove_trusted_address(env: &Env, chain: String, trusted_address: String) {
    let topics = (
        Symbol::new(env, "trusted_address_removed"),
        chain,
        trusted_address,
    );
    env.events().publish(topics, ());
}

pub(crate) fn transfer_ownership(env: &Env, previous_owner: Address, new_owner: Address) {
    let topics = (
        Symbol::new(env, "ownership_transferred"),
        previous_owner,
        new_owner,
    );
    env.events().publish(topics, ());
}
