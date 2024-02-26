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

pub(crate) fn approve_contract_call(
    env: &Env,
    command_id: BytesN<32>,
    source_chain: String,
    source_address: String,
    contract_address: Address,
    payload_hash: BytesN<32>,
) {
    let topics = (
        symbol_short!("approved"),
        command_id,
        contract_address,
        payload_hash,
    );
    env.events().publish(topics, (source_chain, source_address));
}

pub(crate) fn execute_contract_call(env: &Env, command_id: BytesN<32>) {
    let topics = (symbol_short!("executed"), command_id);
    env.events().publish(topics, ());
}

pub(crate) fn execute_command(env: &Env, command_id: BytesN<32>) {
    let topics = (symbol_short!("command"), command_id);
    env.events().publish(topics, ());
}

pub(crate) fn transfer_operatorship(env: &Env, new_operator_set: Bytes) {
    let topics = (symbol_short!("transfer"),);
    env.events().publish(topics, (new_operator_set,));
}
