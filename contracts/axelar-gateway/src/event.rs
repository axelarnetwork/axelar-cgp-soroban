use crate::types::Message;
use soroban_sdk::{Address, Bytes, BytesN, Env, String, Symbol};

pub fn call_contract(
    env: &Env,
    caller: Address,
    destination_chain: String,
    destination_address: String,
    payload: Bytes,
    payload_hash: BytesN<32>,
) {
    let topics = (
        Symbol::new(env, "contract_called"),
        caller,
        destination_chain,
        destination_address,
        payload_hash,
    );
    env.events().publish(topics, payload);
}

pub fn approve_message(env: &Env, message: Message) {
    let topics = (Symbol::new(env, "message_approved"), message);
    env.events().publish(topics, ());
}

pub fn execute_message(env: &Env, message: Message) {
    let topics = (Symbol::new(env, "message_executed"), message);
    env.events().publish(topics, ());
}

pub fn rotate_signers(env: &Env, epoch: u64, signers_hash: BytesN<32>) {
    let topics = (Symbol::new(env, "signers_rotated"), epoch, signers_hash);
    env.events().publish(topics, ());
}
