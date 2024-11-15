#![no_std]

mod abi;
pub mod error;
mod event;
mod storage_types;
mod types;

pub mod contract;

#[cfg(test)]
mod test;

#[cfg(test)]
extern crate std;

pub use contract::InterchainTokenServiceClient;
