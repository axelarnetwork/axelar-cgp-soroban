use soroban_sdk::{Bytes, BytesN, Env, String, Symbol};

pub fn set_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_set"), chain);
    env.events().publish(topics, ());
}

pub fn remove_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_removed"), chain);
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

pub fn interchain_transfer_received(
    env: &Env,
    original_source_chain: String,
    token_id: BytesN<32>,
    source_address: Bytes,
    destination_address: Bytes,
    amount: i128,
    data: Option<Bytes>,
) {
    let topics = (
        Symbol::new(env, "interchain_transfer_received"),
        original_source_chain,
        token_id,
        source_address,
        destination_address,
        amount,
    );
    env.events().publish(topics, (data,));
}
