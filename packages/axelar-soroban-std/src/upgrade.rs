use soroban_sdk::{BytesN, Env, String};

pub trait UpgradeableInterface {
    type Error: Into<soroban_sdk::Error>;
    fn version(env: Env) -> String;
    fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), Self::Error>;
}
