#![no_std]

mod auth;
mod event;
mod storage_types;

pub mod contract;
pub mod error;
pub mod executable;
pub mod types;

#[cfg(all(target_family = "wasm", feature = "testutils"))]
compile_error!("'testutils' feature is not supported on 'wasm' target");

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod test_gateway;

#[cfg(test)]
mod test_auth;

// Allows using std (and its macros) in test modules
#[cfg(test)]
#[macro_use]
extern crate std;

pub use contract::{AxelarGateway, AxelarGatewayClient};
