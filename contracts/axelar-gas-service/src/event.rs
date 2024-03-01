use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, String, U256};

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

pub(crate) fn native_gas_paid_for_contract_call(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    refund_address: Address,
) {
    let topics = (symbol_short!("cc_g_paid"),);
    env.events().publish(
        topics,
        (
            sender,
            destination_chain,
            destination_address,
            env.crypto().keccak256(&payload),
            refund_address,
        ),
    );
}
