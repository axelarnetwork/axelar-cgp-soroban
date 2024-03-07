#![no_std]

pub mod error;
pub mod interface;

pub use interface::AxelarExecutableInterface;

#[cfg(test)]
mod test;
