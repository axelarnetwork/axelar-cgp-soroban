#![no_std]

mod error;
pub mod interface;

pub use interface::AxelarExecutableInterface;

#[cfg(test)]
mod test;
