#![no_std]

mod abi;
pub mod error;
mod event;
mod storage_types;
mod types;

pub mod contract;

#[cfg(test)]
#[macro_use]
extern crate std;

pub use contract::InterchainTokenServiceClient;
