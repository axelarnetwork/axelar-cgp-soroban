use axelar_soroban_std::types::Hash;
use soroban_sdk::{symbol_short, Address, Bytes, Env, String, U256};

pub(crate) fn gas_paid_for_contract_call(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    refund_address: Address,
    token_addr: Address,
    amount: i128,
) {
    let topics = (
        symbol_short!("gas_paid"),
        env.crypto().keccak256(&payload),
        sender,
        destination_chain,
    );
    env.events().publish(
        topics,
        (
            destination_address,
            payload,
            refund_address,
            token_addr,
            amount,
        ),
    );
}

pub(crate) fn refunded(
    env: &Env,
    tx_hash: Hash,
    log_index: U256,
    receiver: &Address,
    token: &Address,
    amount: i128,
) {
    let topics = (symbol_short!("refunded"), tx_hash, log_index, receiver);
    env.events().publish(topics, (token, amount));
}

pub(crate) fn fee_collected(env: &Env, receiver: &Address, token_address: &Address, amount: i128) {
    let topics = (symbol_short!("collected"), receiver, token_address, amount);
    env.events().publish(topics, ());
}
