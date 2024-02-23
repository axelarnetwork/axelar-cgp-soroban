use soroban_sdk::{symbol_short, Address, Env, Symbol, Vec, Bytes, BytesN, String};

pub(crate) fn call_contract(env: &Env, caller: Address, destination_chain: String, destination_address: String, payload: Bytes, payload_hash: BytesN<32>) {
    let topics = (symbol_short!("called"), caller, payload_hash);
    env.events().publish(topics, (destination_chain, destination_address, payload));
}

pub(crate) fn approve_contract_call(env: &Env, caller: Address, destination_chain: String, destination_address: String, payload: Bytes, payload_hash: BytesN<32>) {
    let topics = (symbol_short!("approved"), caller, payload_hash);
    env.events().publish(topics, (destination_chain, destination_address, payload));
}

pub(crate) fn execute_contract_call(env: &Env, command_id: BytesN<32>) {
    let topics = (symbol_short!("executed"), command_id);
    env.events().publish(topics, ());
}
