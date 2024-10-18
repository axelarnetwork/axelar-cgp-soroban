#![no_std]

mod auth;
mod event;
mod storage_types;
pub mod types;

pub mod contract;

#[cfg(all(target_family = "wasm", feature = "testutils"))]
compile_error!("'testutils' feature is not supported on 'wasm' target");

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(test)]
mod test_gateway;

#[cfg(test)]
mod test_auth;
