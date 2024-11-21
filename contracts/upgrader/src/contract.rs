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
        caller: Address,
        contract_address: Address,
        new_version: String,
        new_wasm_hash: BytesN<32>,
        migration_data: soroban_sdk::Vec<Val>,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        // run the upgrade and migration in the context of the given caller,
        // so to the upgradeable contract it looks like the caller invoked those functions directly,
        // and can authenticate them correctly
        env.as_contract(&caller, || {
            ensure_new_version_is_different(&env, &contract_address, &new_version)?;

            // it's safe to map the true return value of the upgrade and migrate functions to (),
            // because we don't care about it, and in case of failure the contract will panic anyway

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
        })
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
    use crate::contract::{Upgrader, UpgraderClient, UPGRADE, VERSION};
    use axelar_soroban_std::UpgradeableInterface;
    use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, Symbol};
    use soroban_sdk::{BytesN, Env, String};

    /// A simple contract to test the upgrader
    #[contract]
    struct DummyContract;

    /// Dummy contract logic before upgrade
    #[contractimpl]
    impl UpgradeableInterface for DummyContract {
        type Error = ContractError;

        fn version(env: Env) -> String {
            String::from_str(&env, "0.1.0")
        }

        fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
            Self::owner(&env).require_auth();

            env.deployer().update_current_contract_wasm(new_wasm_hash);
            Ok(())
        }
    }

    impl DummyContract {
        fn owner(env: &Env) -> Address {
            env.storage().instance().get(&DataKey::Owner).unwrap()
        }
    }

    #[contracttype]
    enum DataKey {
        Data,
        Owner,
    }

    #[contracterror]
    enum ContractError {
        SomeFailure = 1,
    }

    /// Dummy contract logic after upgrade is loaded into WASM_AFTER_UPGRADE
    ///
    /// #[contractimpl]
    /// impl UpgradeableInterface for DummyContract {
    ///     type Error = ContractError;
    ///
    ///     fn version(env: Env) -> String {
    ///         String::from_str(&env, "0.2.0")
    ///     }
    ///
    ///     fn upgrade(env: Env, new_wasm_hash: BytesN<32>) -> Result<(), ContractError> {
    ///         Self::owner(&env).require_auth();
    ///
    ///         env.deployer().update_current_contract_wasm(new_wasm_hash);
    ///         Ok(())
    ///     }
    /// }
    ///
    /// #[contractimpl]
    /// impl DummyContract {
    ///     pub fn migrate(env: Env, migration_data: String) {
    ///         Self::owner(&env).require_auth();
    ///
    ///         env.storage()
    ///             .instance()
    ///             .set(&DataKey::Data, &migration_data);
    ///     }
    ///
    ///     fn owner(env: &Env) -> Address {
    ///         env.storage().instance().get(&DataKey::Owner).unwrap()
    ///     }
    /// }
    const WASM_AFTER_UPGRADE: &[u8] = include_bytes!("testdata/dummy.wasm");

    #[test]
    fn upgrade_and_migrate_are_atomic() {
        let env = Env::default();

        let contract_address = env.register(DummyContract, ());
        let upgrader_address = env.register(Upgrader, ());

        let owner = set_owner(&env, &contract_address);

        let original_version: String = query_version(&env, &contract_address);
        assert_eq!(original_version, String::from_str(&env, "0.1.0"));

        let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
        let expected_data = String::from_str(&env, "migration successful");
        let expected_version = String::from_str(&env, "0.2.0");

        let params = upgrade_call_params(
            &env,
            &upgrader_address,
            &owner,
            &contract_address,
            &expected_version,
            &hash_after_upgrade,
            &expected_data,
        );

        // add the owner to the set of authenticated addresses
        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &params,
        }]);

        UpgraderClient::new(&env, &upgrader_address).upgrade(
            &owner,
            &contract_address,
            &expected_version,
            &hash_after_upgrade,
            &soroban_sdk::vec![&env, expected_data.to_val()],
        );

        // ensure new version is set correctly
        let upgraded_version: String = env.invoke_contract(
            &contract_address,
            &Symbol::new(&env, VERSION),
            soroban_sdk::vec![&env],
        );
        assert_eq!(upgraded_version, expected_version);

        // ensure migration was successful
        env.as_contract(&contract_address, || {
            let data: String = env.storage().instance().get(&DataKey::Data).unwrap();
            assert_eq!(data, expected_data);
        });
    }

    #[test]
    #[should_panic]
    fn upgrade_fails_if_caller_param_does_not_match_actual_caller() {
        let env = Env::default();

        let contract_address = env.register(DummyContract, ());
        let upgrader_address = env.register(Upgrader, ());

        let owner = set_owner(&env, &contract_address);

        let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
        let expected_data = String::from_str(&env, "migration successful");
        let expected_version = String::from_str(&env, "0.2.0");

        let params = upgrade_call_params(
            &env,
            &upgrader_address,
            &owner,
            &contract_address,
            &expected_version,
            &hash_after_upgrade,
            &expected_data,
        );

        // add the owner to the set of authenticated addresses
        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &params,
        }]);

        // should panic: call the upgrade contract as the owner, but pass a wrong address that is not authenticated
        let wrong_param = Address::generate(&env);
        env.as_contract(&owner, || {
            UpgraderClient::new(&env, &upgrader_address).upgrade(
                &wrong_param,
                &contract_address,
                &expected_version,
                &hash_after_upgrade,
                &soroban_sdk::vec![&env, expected_data.to_val()],
            )
        });
    }

    #[test]
    #[should_panic]
    fn upgrade_fails_if_caller_is_not_owner() {
        let env = Env::default();

        let contract_address = env.register(DummyContract, ());
        let upgrader_address = env.register(Upgrader, ());

        let owner = set_owner(&env, &contract_address);

        let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
        let expected_data = String::from_str(&env, "migration successful");
        let expected_version = String::from_str(&env, "0.2.0");

        let params = upgrade_call_params(
            &env,
            &upgrader_address,
            &owner,
            &contract_address,
            &expected_version,
            &hash_after_upgrade,
            &expected_data,
        );

        // add the owner to the set of authenticated addresses
        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &params,
        }]);

        let caller = Address::generate(&env);
        let params = upgrade_call_params(
            &env,
            &upgrader_address,
            &caller,
            &contract_address,
            &expected_version,
            &hash_after_upgrade,
            &expected_data,
        );

        // add the caller to the set of authenticated addresses
        env.mock_auths(&[MockAuth {
            address: &caller,
            invoke: &params,
        }]);

        // should panic: both caller and owner are authenticated, but the upgradeable contract only accepts the owner,
        // and the upgrade contract runs the upgrade in the context of the caller
        env.as_contract(&caller, || {
            UpgraderClient::new(&env, &upgrader_address).upgrade(
                &caller,
                &contract_address,
                &expected_version,
                &hash_after_upgrade,
                &soroban_sdk::vec![&env, expected_data.to_val()],
            )
        });
    }

    #[test]
    #[should_panic]
    fn upgrade_fails_if_caller_does_neither_match_caller_param_not_owner() {
        let env = Env::default();

        let contract_address = env.register(DummyContract, ());
        let upgrader_address = env.register(Upgrader, ());

        let owner = set_owner(&env, &contract_address);

        let hash_after_upgrade = env.deployer().upload_contract_wasm(WASM_AFTER_UPGRADE);
        let expected_data = String::from_str(&env, "migration successful");
        let expected_version = String::from_str(&env, "0.2.0");

        let caller = Address::generate(&env);
        let wrong_caller_param = Address::generate(&env);

        // authenticate all addresses
        [&owner, &caller, &wrong_caller_param]
            .iter()
            .for_each(|address| {
                let params = upgrade_call_params(
                    &env,
                    &upgrader_address,
                    address,
                    &contract_address,
                    &expected_version,
                    &hash_after_upgrade,
                    &expected_data,
                );

                // add the owner to the set of authenticated addresses
                env.mock_auths(&[MockAuth {
                    address,
                    invoke: &params,
                }])
            });

        // should panic: while all addresses are authenticated, they must all match for the invocation to succeed
        env.as_contract(&caller, || {
            UpgraderClient::new(&env, &upgrader_address).upgrade(
                &wrong_caller_param,
                &contract_address,
                &expected_version,
                &hash_after_upgrade,
                &soroban_sdk::vec![&env, expected_data.to_val()],
            )
        });
    }

    fn upgrade_call_params<'a>(
        env: &Env,
        upgrader_address: &'a Address,
        owner: &'a Address,
        contract_address: &'a Address,
        expected_version: &'a String,
        hash_after_upgrade: &'a BytesN<32>,
        expected_data: &'a String,
    ) -> MockAuthInvoke<'a> {
        MockAuthInvoke {
            contract: upgrader_address,
            fn_name: UPGRADE,
            args: soroban_sdk::vec![
                &env,
                owner.to_val(),
                contract_address.to_val(),
                expected_version.to_val(),
                hash_after_upgrade.to_val(),
                soroban_sdk::vec![&env, expected_data.to_val()].to_val()
            ],
            sub_invokes: &[],
        }
    }

    fn query_version(env: &Env, contract_address: &Address) -> String {
        env.invoke_contract(
            contract_address,
            &Symbol::new(env, VERSION),
            soroban_sdk::vec![&env],
        )
    }

    fn set_owner(env: &Env, contract_address: &Address) -> Address {
        let owner = Address::generate(env);
        env.as_contract(contract_address, || {
            env.storage().instance().set(&DataKey::Owner, &owner);
        });
        owner
    }
}
