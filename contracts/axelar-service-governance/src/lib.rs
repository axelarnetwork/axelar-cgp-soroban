#![no_std]

pub mod contract;
mod error;
mod event;
mod storage_types;
mod types;

#[cfg(test)]
mod test;

pub use contract::AxelarServiceGovernanceClient;
