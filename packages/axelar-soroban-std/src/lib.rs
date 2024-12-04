#![no_std]
#[cfg(test)]
extern crate std;
// required by goldie

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub mod traits;

pub mod types;

pub mod error;

pub mod ttl;

pub mod contract_traits;
