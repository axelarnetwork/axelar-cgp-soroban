use soroban_sdk::{Address, Bytes, Env, String, Symbol};

pub fn gas_paid_for_contract_call(
    env: &Env,
    sender: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    refund_address: Address,
    token_address: Address,
    token_amount: i128,
) {
    let topics = (
        Symbol::new(env, "gas_paid"),
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
            token_address,
            token_amount,
        ),
    );
}

pub fn gas_added(
    env: &Env,
    message_id: String,
    token_address: Address,
    token_amount: i128,
    refund_address: Address,
) {
    let topics = (
        Symbol::new(env, "gas_added"),
        message_id,
        token_address,
        token_amount,
        refund_address,
    );
    env.events().publish(topics, ());
}

pub fn refunded(
    env: &Env,
    message_id: String,
    receiver: Address,
    token_address: Address,
    token_amount: i128,
) {
    let topics = (
        Symbol::new(env, "gas_refunded"),
        message_id,
        receiver,
        token_address,
        token_amount,
    );
    env.events().publish(topics, ());
}

pub fn fee_collected(env: &Env, receiver: Address, token_address: Address, token_amount: i128) {
    let topics = (
        Symbol::new(env, "gas_collected"),
        receiver,
        token_address,
        token_amount,
    );
    env.events().publish(topics, ());
}
