#![no_std]

mod error;
mod event;
mod storage_types;

pub mod contract;

#[cfg(test)]
mod test;

pub use contract::InterchainTokenServiceClient;
