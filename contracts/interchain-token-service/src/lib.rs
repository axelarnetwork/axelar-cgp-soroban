#![no_std]

#[cfg(any(test, feature = "testutils"))]
extern crate std;

pub mod error;
pub mod event;
pub mod executable;
mod interface;
pub mod types;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "library", not(feature = "testutils")))] {
        pub use interface::{InterchainTokenServiceClient, InterchainTokenServiceInterface};
    } else {
        mod abi;
        mod storage_types;
        mod token_handler;
        mod contract;
        pub use contract::{InterchainTokenService, InterchainTokenServiceClient};
    }
}
