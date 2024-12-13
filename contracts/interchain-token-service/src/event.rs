use core::fmt::Debug;

use axelar_soroban_std::events::Event;
use soroban_sdk::{contracttype, Bytes, BytesN, Env, IntoVal, String, Symbol, Topics, Val};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransferSent {
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

impl Event for InterchainTransferSent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (Symbol::new(env, "interchain_transfer_sent"), self.token_id.to_val(), self.source_address.to_val(), self.destination_address.to_val(), self.amount)
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        let data_hash = self.data.clone().map_or_else(|| BytesN::<32>::from_array(env, &[0; 32]), |data| { env.crypto().keccak256(&data).into() });
        (data_hash,)
    }
}

pub fn set_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_set"), chain);
    env.events().publish(topics, ());
}

pub fn remove_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_removed"), chain);
    env.events().publish(topics, ());
}

pub fn executed(
    env: &Env,
    source_chain: String,
    message_id: String,
    source_address: String,
    payload: Bytes,
) {
    let topics = (
        Symbol::new(env, "executed"),
        source_chain,
        message_id,
        source_address,
    );
    env.events().publish(topics, (payload,));
}

pub fn interchain_transfer_received(
    env: &Env,
    original_source_chain: String,
    token_id: BytesN<32>,
    source_address: Bytes,
    destination_address: Bytes,
    amount: i128,
    data: Option<Bytes>,
) {
    let topics = (
        Symbol::new(env, "interchain_transfer_received"),
        original_source_chain,
        token_id,
        source_address,
        destination_address,
        amount,
    );
    env.events().publish(topics, (data,));
}
