#![no_std]

#[cfg(any(test, feature = "testutils"))]
pub mod testutils;

#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

#[cfg(any(test, feature = "testutils"))]
pub mod traits;

pub mod error;
