#![no_std]
#[cfg(test)]
extern crate alloc;

mod contract;
pub mod error;

pub use contract::{Upgrader, UpgraderClient};
