#![no_std]

pub mod error;
mod event;
mod storage_types;

pub mod contract;

pub use contract::GmpExampleClient;
