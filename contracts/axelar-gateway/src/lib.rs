#![no_std]

mod gateway;
mod admin;

pub use crate::gateway::GatewayClient;

#[cfg(test)]
mod test;
