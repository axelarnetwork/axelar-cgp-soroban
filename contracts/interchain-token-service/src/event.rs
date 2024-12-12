use axelar_soroban_std::events::Event;
#[cfg(any(test, feature = "testutils"))]
use axelar_soroban_std::impl_event_testutils;
use core::fmt::Debug;
use soroban_sdk::{Bytes, BytesN, Env, IntoVal, String, Symbol, Topics, Val};

pub fn set_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_set"), chain);
    env.events().publish(topics, ());
}

pub fn remove_trusted_chain(env: &Env, chain: String) {
    let topics = (Symbol::new(env, "trusted_chain_removed"), chain);
    env.events().publish(topics, ());
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTransferReceivedEvent {
    pub original_source_chain: String,
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

impl Event for InterchainTransferReceivedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_transfer_received"),
            self.original_source_chain.as_val(),
            self.token_id.to_val(),
            self.source_address.to_val(),
            self.destination_address.to_val(),
            self.amount,
        )
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
        (self.data.clone(),)
    }
}

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTransferReceivedEvent,
    (Symbol, String, BytesN<32>, Bytes, Bytes, i128),
    (Option<Bytes>)
);

// pub fn interchain_transfer_received(
//     env: &Env,
//     original_source_chain: String,
//     token_id: BytesN<32>,
//     source_address: Bytes,
//     destination_address: Bytes,
//     amount: i128,
//     data: Option<Bytes>,
// ) {
//     let topics = (
//         Symbol::new(env, "interchain_transfer_received"),
//         original_source_chain,
//         token_id,
//         source_address,
//         destination_address,
//         amount,
//     );
//     env.events().publish(topics, (data,));
// }
