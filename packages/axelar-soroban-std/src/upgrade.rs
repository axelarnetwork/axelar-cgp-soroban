use soroban_sdk::{BytesN, Env, String};

pub trait UpgradeableInterface {
    type Error: Into<soroban_sdk::Error>;

    /// Returns the current version of the contract.
    fn version(env: &Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) -> Result<(), Self::Error>;
}
