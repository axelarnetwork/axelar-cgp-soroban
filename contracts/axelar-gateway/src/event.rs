use axelar_soroban_interfaces::types::{Message, WeightedSigners};
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, String};

pub(crate) fn call_contract(
    env: &Env,
    caller: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    payload_hash: BytesN<32>,
) {
    let topics = (symbol_short!("called"), caller, payload_hash);
    env.events()
        .publish(topics, (destination_chain, destination_address, payload));
}

pub(crate) fn approve_message(env: &Env, message: Message) {
    let topics = (
        symbol_short!("approved"),
        message.message_id,
        message.contract_address,
        message.payload_hash,
    );
    env.events()
        .publish(topics, (message.source_chain, message.source_address));
}

pub(crate) fn execute_contract_call(env: &Env, message: Message) {
    let topics = (
        symbol_short!("executed"),
        message.message_id,
        message.contract_address,
        message.payload_hash,
    );
    env.events()
        .publish(topics, (message.source_chain, message.source_address));
}

pub(crate) fn rotate_signers(env: &Env, signers: WeightedSigners) {
    let topics = (symbol_short!("rotated"),);
    env.events().publish(topics, signers);
}

pub(crate) fn transfer_operatorship(env: &Env, previous_operator: Address, new_operator: Address) {
    let topics = (
        String::from_str(env, "transferred"),
        previous_operator,
        new_operator,
    );
    env.events().publish(topics, ());
}
