use axelar_soroban_interfaces::types::Message;
use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, String};

pub(crate) fn call_contract(
    env: &Env,
    caller: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    payload_hash: BytesN<32>,
) {
    let topics = (
        symbol_short!("called"),
        caller,
        destination_chain,
        destination_address,
        payload_hash,
    );
    env.events().publish(topics, payload);
}

pub(crate) fn approve_message(env: &Env, message: Message) {
    let topics = (symbol_short!("approved"),);
    env.events().publish(topics, message);
}

pub(crate) fn execute_contract_call(env: &Env, message: Message) {
    let topics = (symbol_short!("executed"),);
    env.events().publish(topics, message);
}

pub(crate) fn rotate_signers(env: &Env, epoch: u64, signers_hash: BytesN<32>) {
    let topics = (symbol_short!("rotated"), epoch, signers_hash);
    env.events().publish(topics, ());
}

pub(crate) fn transfer_operatorship(env: &Env, previous_operator: Address, new_operator: Address) {
    let topics = (
        String::from_str(env, "transferred"),
        previous_operator,
        new_operator,
    );
    env.events().publish(topics, ());
}
