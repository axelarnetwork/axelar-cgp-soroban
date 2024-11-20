#![allow(dead_code)]

use crate::error::ContractError;
use axelar_soroban_std::traits::ThenOk;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Val};

const MIGRATE: &str = "migrate";
const UPGRADE: &str = "upgrade";
const VERSION: &str = "version";

#[contract]
pub struct Upgrader;

#[contractimpl]
impl Upgrader {
    pub fn __constructor(_env: Env) {}

    pub fn upgrade(
        env: Env,
        contract_address: Address,
        new_version: String,
        new_wasm_hash: BytesN<32>,
        migration_data: soroban_sdk::Vec<Val>,
    ) -> Result<(), ContractError> {
        ensure_new_version_is_different(&env, &contract_address, &new_version)?;

        env.invoke_contract::<()>(
            &contract_address,
            &Symbol::new(&env, UPGRADE),
            soroban_sdk::vec![&env, new_wasm_hash.into()],
        );
        env.invoke_contract::<()>(
            &contract_address,
            &Symbol::new(&env, MIGRATE),
            migration_data,
        );

        ensure_new_version_matches_expected(&env, &contract_address, &new_version)
    }
}

fn ensure_new_version_is_different(
    env: &Env,
    contract_address: &Address,
    new_version: &String,
) -> Result<(), ContractError> {
    let string = current_version(env, contract_address);
    let no_match = string != *new_version;
    no_match.then_ok((), ContractError::SameVersion)
}

fn ensure_new_version_matches_expected(
    env: &Env,
    contract_address: &Address,
    new_version: &String,
) -> Result<(), ContractError> {
    let versions_match = current_version(env, contract_address) == *new_version;
    versions_match.then_ok((), ContractError::VersionMismatch)
}

fn current_version(env: &Env, contract_address: &Address) -> String {
    env.invoke_contract(
        contract_address,
        &Symbol::new(env, VERSION),
        soroban_sdk::vec![env],
    )
}

#[cfg(test)]
mod tests {
    use crate::contract::{Upgrader, UpgraderClient};
    use soroban_sdk::{contract, contractimpl, contracttype};
    use soroban_sdk::{BytesN, Env, String};

    const WASM_AFTER_UPGRADE: &[u8] = include_bytes!("testdata/upgraded_dummy_contract.wasm");

    #[contract]
    struct DummyContract;

    /// Dummy contract logic before upgrade
    #[contractimpl]
    impl DummyContract {
        pub fn version(env: Env) -> String {
            String::from_str(&env, "0.1.0")
        }

        pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> () {
            env.deployer().update_current_contract_wasm(new_wasm_hash)
        }
    }

    // Dummy contract logic after upgrade (as is loaded into WASM_AFTER_UPGRADE)

    // #[contractimpl]
    // impl DummyContract {
    //     pub fn version(env: Env) -> String {
    //         String::from_str(&env, "0.2.0")
    //     }
    //
    //     pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> () {
    //         env.deployer().update_current_contract_wasm(new_wasm_hash)
    //     }
    //
    //     pub fn migrate(env: Env, migration_data: String) {
    //         env.storage().instance().set(&DataKey::Data, &migration_data);
    //     }
    // }

    #[contracttype]
    enum DataKey {
        Data,
    }

    #[test]
    fn test() {
        let env = Env::default();

        let contract_address = env.register(DummyContract, ());

        let upgrader_address = env.register(Upgrader, ());

        let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
        let client = UpgraderClient::new(&env, &upgrader_address);

        let expected_data = String::from_str(&env, "migration successful");
        client.upgrade(
            &contract_address,
            &String::from_str(&env, "0.2.0"),
            &hash_after_upgrade,
            &soroban_sdk::vec![&env, expected_data.to_val()],
        );

        env.as_contract(&contract_address, || {
            let data: String = env.storage().instance().get(&DataKey::Data).unwrap();
            assert_eq!(data, expected_data);
        });
    }
}
