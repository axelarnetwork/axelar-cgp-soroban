//! Test-only mirrors of the SEP-41 events emitted by the Soroban token SDK.
//!
//! The contract itself never emits these structs: its operations emit the standard SDK events
//! (`TransferWithAmountOnly`, `MintWithAmountOnly`, `Approve`, `Burn`). These definitions exist
//! solely so tests can format and assert on those emitted events, and are kept out of
//! `crate::event` so that module only holds events the contract actually emits.

use stellar_axelar_std::{Address, IntoEvent};

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct TransferEvent {
    pub from: Address,
    pub to: Address,
    #[datum]
    pub amount: i128,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct MintEvent {
    pub to: Address,
    #[datum]
    pub amount: i128,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct ApproveEvent {
    pub owner: Address,
    pub spender: Address,
    #[data]
    pub amount: i128,
    #[data]
    pub expiration_ledger: u32,
}

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct BurnEvent {
    pub from: Address,
    #[datum]
    pub amount: i128,
}
