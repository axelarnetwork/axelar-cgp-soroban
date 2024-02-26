use soroban_sdk::{symbol_short, Address, BytesN, Env, U256};

pub(crate) fn refunded(env: &Env, tx_hash: BytesN<32>, log_index: U256, receiver: Address, token: Address, amount: i128) {
    let topics = (symbol_short!("refunded"), tx_hash, log_index);
    env.events().publish(topics, (receiver, token, amount));
}