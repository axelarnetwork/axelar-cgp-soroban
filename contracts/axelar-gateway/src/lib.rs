#![no_std]

mod gateway;
mod admin;

pub use crate::gateway::GatewayClient;
pub mod interface;

#[cfg(test)]
mod test;
