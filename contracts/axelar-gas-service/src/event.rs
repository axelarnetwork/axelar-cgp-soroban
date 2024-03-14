use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, String, U256};

pub(crate) fn gas_paid_for_contract_call(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    refund_address: Address,
) {
    let topics = (symbol_short!("gas_paid"), env.crypto().keccak256(&payload));
    env.events().publish(
        topics,
        (
            sender,
            destination_chain,
            destination_address,
            payload,
            refund_address,
        ),
    );
}

pub(crate) fn refunded(
    env: &Env,
    tx_hash: BytesN<32>,
    log_index: U256,
    receiver: &Address,
    token: &Address,
    amount: i128,
) {
    let topics = (symbol_short!("refunded"), tx_hash, log_index);
    env.events().publish(topics, (receiver, token, amount));
}

pub(crate) fn fee_collected(env: &Env, receiver: &Address, token_address: &Address, amount: i128) {
    let topics = (symbol_short!("coll_fees"),);
    env.events()
        .publish(topics, (receiver, token_address, amount));
}
