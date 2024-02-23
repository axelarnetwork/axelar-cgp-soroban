#![no_std]

mod gateway;
mod admin;

pub use crate::gateway::GatewayClient;
pub mod interface;
mod contract;

#[cfg(test)]
mod test;
