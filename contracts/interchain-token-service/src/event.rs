use core::fmt::Debug;

use axelar_soroban_std::events::Event;
#[cfg(any(test, feature = "testutils"))]
use axelar_soroban_std::impl_event_testutils;
use soroban_sdk::{
    contracttype, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Topics, Val,
};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterchainTransferSentEvent {
    pub token_id: BytesN<32>,
    pub source_address: Address,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

impl Event for InterchainTransferSentEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_transfer_sent"),
            self.token_id.to_val(),
            self.source_address.to_val(),
            self.destination_address.to_val(),
            self.amount,
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        let data_hash = self.data.clone().map_or_else(
            || BytesN::<32>::from_array(env, &[0; 32]),
            |data| env.crypto().keccak256(&data).into(),
        );
        (data_hash,)
    }
}

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTransferSentEvent,
    (Symbol, BytesN<32>, Bytes, Address, i128),
    (BytesN<32>)
);

#[derive(Debug, PartialEq, Eq)]
pub struct TrustedChainSetEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TrustedChainRemovedEvent {
    pub chain: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InterchainTransferReceivedEvent {
    pub source_chain: String,
    pub token_id: BytesN<32>,
    pub source_address: Bytes,
    pub destination_address: Bytes,
    pub amount: i128,
    pub data: Option<Bytes>,
}

impl Event for TrustedChainSetEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (Symbol::new(env, "trusted_chain_set"), self.chain.to_val())
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

impl Event for TrustedChainRemovedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "trusted_chain_removed"),
            self.chain.to_val(),
        )
    }

    fn data(&self, env: &Env) -> impl IntoVal<Env, Val> + Debug {
        Vec::<Val>::new(env)
    }
}

impl Event for InterchainTransferReceivedEvent {
    fn topics(&self, env: &Env) -> impl Topics + Debug {
        (
            Symbol::new(env, "interchain_transfer_received"),
            self.source_chain.as_val(),
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
impl_event_testutils!(TrustedChainSetEvent, (Symbol, String), ());

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(TrustedChainRemovedEvent, (Symbol, String), ());

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(
    InterchainTransferReceivedEvent,
    (Symbol, String, BytesN<32>, Bytes, Bytes, i128),
    (Option<Bytes>)
);
