#![no_std]

pub mod error;

mod interface;

// Allows using std (and its macros) in test modules
#[cfg(test)]
#[macro_use]
extern crate std;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenClient, InterchainTokenInterface};
    } else {
        mod event;
        mod storage_types;

        pub mod contract;
        pub use contract::{InterchainToken, InterchainTokenClient};
    }
}
