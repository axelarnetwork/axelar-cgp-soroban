pub mod contract;
pub mod error;
mod event;
mod storage_types;
mod utils;

// Allows using std (and its macros) in test modules
#[cfg(test)]
#[macro_use]
extern crate std;

pub use crate::contract::InterchainTokenClient;