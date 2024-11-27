use crate::ensure;
use soroban_sdk::{
    contractclient, symbol_short, Address, BytesN, ConversionError, Env, FromVal, IntoVal, String,
    Topics, TryFromVal, Val, Vec,
};

#[contractclient(name = "OwnershipClient")]
pub trait OwnershipInterface {
    fn owner(env: &Env) -> Address;
}

#[contractclient(name = "UpgradeableClient")]
pub trait UpgradeableInterface: OwnershipInterface {
    /// Returns the current version of the contract.
    fn version(env: &Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>);
}

pub trait MigratableInterface: UpgradeableInterface {
    /// Data needed during the migration. Each contract can define its own data type.
    type MigrationData: FromVal<Env, Val>;
    /// Error type returned if the migration fails.
    type Error: Into<soroban_sdk::Error>;

    /// Migrates contract state after upgrading the contract code.
    fn migrate(env: &Env, migration_data: Self::MigrationData) -> Result<(), Self::Error>;
}

/// Default implementation of the [OwnershipInterface] trait.
pub fn owner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&storage::DataKey::SharedInterfaces_Owner)
        .expect("owner must be set during contract construction")
}

/// Default implementation accompanying the [OwnershipInterface] trait. This should never be part of a contract interface,
/// but allows contracts internally to set the owner.
pub fn set_owner(env: &Env, owner: &Address) {
    env.storage()
        .instance()
        .set(&storage::DataKey::SharedInterfaces_Owner, owner);
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then upgrades the contract to a new WASM hash and prepares it for migration.
pub fn upgrade<T: OwnershipInterface>(env: &Env, new_wasm_hash: BytesN<32>) {
    T::owner(env).require_auth();

    env.deployer().update_current_contract_wasm(new_wasm_hash);
    start_migration(env);
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then runs the custom_migration and finalizes the migration.
/// An event is emitted when the migration, and with it the overall upgrade, is complete.
/// Migration can only be run once, after the [upgrade] function has been called.
pub fn migrate<T: UpgradeableInterface>(
    env: &Env,
    custom_migration: impl FnOnce(),
) -> Result<(), MigrationError> {
    T::owner(env).require_auth();

    ensure_is_migrating(env)?;

    custom_migration();
    complete_migration(env);

    emit_event_upgraded(
        env,
        UpgradedEvent {
            version: T::version(env),
        },
    );

    Ok(())
}

fn emit_event_upgraded(env: &Env, event: UpgradedEvent) {
    env.events().publish(UpgradedEvent::topic(), event.data());
}

fn start_migration(env: &Env) {
    env.storage()
        .instance()
        .set(&storage::DataKey::SharedInterfaces_Migrating, &());
}

fn ensure_is_migrating(env: &Env) -> Result<(), MigrationError> {
    ensure!(
        env.storage()
            .instance()
            .has(&storage::DataKey::SharedInterfaces_Migrating),
        MigrationError::NotAllowed
    );

    Ok(())
}
fn complete_migration(env: &Env) {
    env.storage()
        .instance()
        .remove(&storage::DataKey::SharedInterfaces_Migrating);
}

#[derive(Debug)]
pub struct UpgradedEvent {
    version: String,
}

impl UpgradedEvent {
    pub fn topic() -> impl Topics {
        (symbol_short!("upgraded"),)
    }

    pub fn data(&self) -> impl IntoVal<Env, Val> {
        (self.version.to_val(),)
    }
}

impl TryFromVal<Env, (Address, Vec<Val>, Val)> for UpgradedEvent {
    type Error = ConversionError;

    fn try_from_val(
        env: &Env,
        (_address, topics, data): &(Address, Vec<Val>, Val),
    ) -> Result<Self, Self::Error> {
        ensure!(topics.eq(&Self::topic().into_val(env)), ConversionError);

        let v: Vec<Val> = Vec::try_from_val(env, data)?;
        String::try_from_val(env, &v.first().ok_or(ConversionError)?)
            .map(|version| Self { version })
    }
}

// submodule to encapsulate the disabled linting
mod storage {
    // linting is disabled for the enum variant names on purpose, so we can define names that would otherwise be invalid.
    // This way, if a contract that implements a shared interface defines a variant with the same name, the linter will
    // complain about it.
    #![allow(non_camel_case_types)]

    use soroban_sdk::contracttype;

    #[contracttype]
    /// Variants do not follow the naming convention of other variants to let the linter help to avoid
    /// collisions with contract types defined in other contracts that implement a shared interface.
    pub enum DataKey {
        SharedInterfaces_Migrating,
        SharedInterfaces_Owner,
    }
}

pub enum MigrationError {
    NotAllowed,
}

#[cfg(test)]
mod test {
    use crate::shared_interfaces::{OwnershipClient, UpgradeableClient, UpgradedEvent};
    use crate::{shared_interfaces, testdata};
    use std::format;

    use crate::testdata::contract::ContractClient;
    use soroban_sdk::testutils::{Address as _, Events, MockAuth, MockAuthInvoke};
    use soroban_sdk::{contracttype, Address, BytesN, Env, String, TryFromVal, TryIntoVal};

    const WASM: &[u8] = include_bytes!("testdata/contract.wasm");

    #[test]
    fn contracttype_enum_name_is_irrelevant_for_key_collision() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        env.as_contract(&contract_id, || {
            assert!(!env
                .storage()
                .instance()
                .has(&testdata::contract::DataKey::Migrating));
            assert!(!env.storage().instance().has(&DataKey2::Migrating));

            env.storage()
                .instance()
                .set(&testdata::contract::DataKey::Migrating, &());

            assert!(env
                .storage()
                .instance()
                .has(&testdata::contract::DataKey::Migrating));
            assert!(env.storage().instance().has(&DataKey2::Migrating));
        });
    }

    #[test]
    #[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
    fn owner_panics_if_owner_not_set() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        OwnershipClient::new(&env, &contract_id).owner();
    }

    #[test]
    fn owner_returns_correct_owner_when_set() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        assert_eq!(OwnershipClient::new(&env, &contract_id).owner(), owner);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(WasmVm, InvalidAction)")]
    fn upgrade_panics_if_owner_not_set() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn upgrade_panics_if_owner_not_authenticated() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);
    }

    #[test]
    fn upgrade_succeeds_if_owner_is_authenticated() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &upgrade_auth(&env, &contract_id, &hash),
        }]);

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Auth, InvalidAction)")]
    fn migrate_panics_if_owner_not_authenticated() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &upgrade_auth(&env, &contract_id, &hash),
        }]);

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);

        ContractClient::new(&env, &contract_id).migrate(&());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn migrate_panics_if_not_called_after_upgrade() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &migrate_auth(&env, &contract_id),
        }]);

        ContractClient::new(&env, &contract_id).migrate(&());
    }

    #[test]
    fn migrate_succeeds_if_owner_is_authenticated_and_called_after_upgrade() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &upgrade_auth(&env, &contract_id, &hash),
        }]);

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);

        let client = ContractClient::new(&env, &contract_id);
        assert!(client.migration_data().is_none());

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &migrate_auth(&env, &contract_id),
        }]);

        client.migrate(&());

        assert_eq!(
            client.migration_data(),
            Some(String::from_str(&env, "migrated"))
        );

        let event = env
            .events()
            .all()
            .iter()
            .find_map(|event| UpgradedEvent::try_from_val(&env, &event).ok());
        goldie::assert!(format!("{:?}", event))
    }

    // Because migration happens on a contract loaded from WASM, code coverage analysis doesn't recognize
    // the migration code as covered. This test repeats the migration test with mocked setup
    #[test]
    fn simulate_migration_for_code_coverage() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
            shared_interfaces::start_migration(&env);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &migrate_auth(&env, &contract_id),
        }]);

        ContractClient::new(&env, &contract_id).migrate(&());
    }

    #[test]
    #[should_panic(expected = "HostError: Error(Contract, #1)")]
    fn migrate_panics_if_called_twice() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        env.mock_auths(&[MockAuth {
            address: &owner,
            invoke: &upgrade_auth(&env, &contract_id, &hash),
        }]);

        UpgradeableClient::new(&env, &contract_id).upgrade(&hash);

        let migrate_auth = &[MockAuth {
            address: &owner,
            invoke: &migrate_auth(&env, &contract_id),
        }];
        env.mock_auths(migrate_auth);

        let client = ContractClient::new(&env, &contract_id);
        client.migrate(&());

        env.mock_auths(migrate_auth);
        client.migrate(&());
    }

    #[contracttype]
    enum DataKey2 {
        Migrating,
    }

    fn upgrade_auth<'a>(
        env: &Env,
        contract_id: &'a Address,
        hash: &'a BytesN<32>,
    ) -> MockAuthInvoke<'a> {
        MockAuthInvoke {
            contract: contract_id,
            fn_name: "upgrade",
            args: soroban_sdk::vec![&env, hash.to_val()],
            sub_invokes: &[],
        }
    }

    fn migrate_auth<'a>(env: &Env, contract_id: &'a Address) -> MockAuthInvoke<'a> {
        MockAuthInvoke {
            contract: contract_id,
            fn_name: "migrate",
            args: soroban_sdk::vec![&env, ().try_into_val(env).unwrap()],
            sub_invokes: &[],
        }
    }
}
