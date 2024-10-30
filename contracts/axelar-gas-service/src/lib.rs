#![no_std]

mod event;

pub mod storage_types;
pub mod contract;
pub mod error;

pub use contract::AxelarGasServiceClient;
