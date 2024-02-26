use soroban_sdk::{symbol_short, Address, Bytes, BytesN, Env, String, Symbol, Vec};

use crate::types::WeightedSigners;

pub(crate) fn transfer_operatorship(
    env: &Env,
    signer_set: WeightedSigners,
    signer_set_hash: BytesN<32>,
) {
    let topics = (symbol_short!("transfer"), signer_set_hash);
    env.events()
        .publish(topics, (signer_set,)); // TODO: use a tuple or the type directly?
}
