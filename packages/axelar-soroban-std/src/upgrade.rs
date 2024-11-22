use soroban_sdk::{contractclient, BytesN, Env, String};

#[contractclient(name = "UpgradeableClient")]
pub trait UpgradeableInterface {
    /// Returns the current version of the contract.
    fn version(env: Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>);
}
