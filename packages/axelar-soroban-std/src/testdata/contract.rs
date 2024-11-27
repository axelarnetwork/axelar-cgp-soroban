use crate::shared_interfaces;
use crate::shared_interfaces::{MigratableInterface, OwnableInterface, UpgradableInterface};
use soroban_sdk::testutils::arbitrary::std;
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, Address, BytesN, Env, String,
};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(_env: Env) {}

    pub fn migration_data(env: &Env) -> Option<String> {
        env.storage().instance().get(&DataKey::Data)
    }

    fn run_migration(env: &Env, _migration_data: ()) {
        env.storage()
            .instance()
            .set(&DataKey::Data, &String::from_str(env, "migrated"));
    }
}

#[contractimpl]
impl MigratableInterface for Contract {
    type MigrationData = ();
    type Error = ContractError;

    fn migrate(env: &Env, migration_data: ()) -> Result<(), ContractError> {
        shared_interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::SomeFailure)
    }
}

#[contractimpl]
impl OwnableInterface for Contract {
    fn owner(env: &Env) -> Address {
        shared_interfaces::owner(env)
    }
}

#[contractimpl]
impl UpgradableInterface for Contract {
    fn version(env: &Env) -> String {
        String::from_str(env, "0.1.0")
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        shared_interfaces::upgrade::<Self>(env, new_wasm_hash);
    }
}

#[contracttype]
pub enum DataKey {
    Data,
    Migrating,
}

#[contracterror]
pub enum ContractError {
    SomeFailure = 1,
}
