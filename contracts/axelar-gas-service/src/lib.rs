#![no_std]

pub mod contract;
mod event;
mod storage_types;
mod error;

#[cfg(test)]
mod test;

pub use contract::AxelarGasServiceClient;
