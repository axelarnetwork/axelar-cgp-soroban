#![no_std]
#[cfg(any(test, feature = "testutils"))]
extern crate std;
// required by goldie

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub mod traits;

pub mod types;

pub mod error;

pub mod shared_interfaces;

pub mod events;

#[cfg(test)]
mod testdata;
