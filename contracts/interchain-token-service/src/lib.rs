#![no_std]

pub mod error;
mod event;
mod storage_types;
mod types;

pub mod contract;

pub use contract::InterchainTokenServiceClient;
