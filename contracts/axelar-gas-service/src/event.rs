use axelar_soroban_std::types::Token;
use soroban_sdk::{Address, Bytes, Env, String, Symbol};

pub(crate) fn gas_paid_for_contract_call(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    refund_address: Address,
    token: Token,
) {
    let topics = (
        Symbol::new(env, "gas_paid"),
        env.crypto().keccak256(&payload),
        sender,
        destination_chain,
    );
    env.events().publish(
        topics,
        (destination_address, payload, refund_address, token),
    );
}

pub(crate) fn refunded(env: &Env, message_id: String, receiver: Address, token: Token) {
    let topics = (
        Symbol::new(env, "gas_refunded"),
        message_id,
        receiver,
        token,
    );
    env.events().publish(topics, ());
}

pub(crate) fn fee_collected(env: &Env, receiver: Address, token: Token) {
    let topics = (Symbol::new(env, "gas_collected"), receiver, token);
    env.events().publish(topics, ());
}
