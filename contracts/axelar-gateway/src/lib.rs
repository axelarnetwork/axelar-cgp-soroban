#![no_std]

mod gateway;
mod admin;

pub use crate::gateway::GatewayClient;
pub mod interface;
mod event;
mod error;
mod storage_types;
pub mod contract;

#[cfg(test)]
mod test;

pub use contract::AxelarGatewayClient;
