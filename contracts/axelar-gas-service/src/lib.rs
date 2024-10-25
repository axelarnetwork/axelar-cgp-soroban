#![no_std]

mod event;
mod storage_types;

pub mod contract;
pub mod error;

#[cfg(test)]
mod test;

pub use contract::AxelarGasServiceClient;
