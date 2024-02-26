#![no_std]

pub mod interface;
mod types;
mod event;
mod storage_types;
pub mod contract;

#[cfg(test)]
mod test;

pub use contract::AxelarGasService;
