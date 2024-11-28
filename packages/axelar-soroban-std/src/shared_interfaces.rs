use crate::ensure;
use crate::events::Event;
use soroban_sdk::{
    contractclient, symbol_short, Address, BytesN, ConversionError, Env, FromVal, IntoVal, String,
    Topics, TryFromVal, Val, Vec,
};

#[contractclient(name = "OwnershipClient")]
pub trait OwnableInterface {
    fn owner(env: &Env) -> Address;
}

#[contractclient(name = "UpgradableClient")]
pub trait UpgradableInterface: OwnableInterface {
    /// Returns the current version of the contract.
    fn version(env: &Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>);
}

pub trait MigratableInterface: UpgradableInterface {
    /// Data needed during the migration. Each contract can define its own data type.
    type MigrationData: FromVal<Env, Val>;
    /// Error type returned if the migration fails.
    type Error: Into<soroban_sdk::Error>;

    /// Migrates contract state after upgrading the contract code.
    fn migrate(env: &Env, migration_data: Self::MigrationData) -> Result<(), Self::Error>;
}

/// Default implementation of the [OwnableInterface] trait.
pub fn owner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&storage::DataKey::SharedInterfaces_Owner)
        .expect("owner must be set during contract construction")
}

/// Default implementation accompanying the [OwnableInterface] trait. This should never be part of a contract interface,
/// but allows contracts internally to set the owner.
pub fn set_owner(env: &Env, owner: &Address) {
    env.storage()
        .instance()
        .set(&storage::DataKey::SharedInterfaces_Owner, owner);
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then upgrades the contract to a new WASM hash and prepares it for migration.
pub fn upgrade<T: OwnableInterface>(env: &Env, new_wasm_hash: BytesN<32>) {
    T::owner(env).require_auth();

    env.deployer().update_current_contract_wasm(new_wasm_hash);
    start_migration(env);
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then runs the custom_migration and finalizes the migration.
/// An event is emitted when the migration, and with it the overall upgrade, is complete.
/// Migration can only be run once, after the [upgrade] function has been called.
pub fn migrate<T: UpgradableInterface>(
    env: &Env,
    custom_migration: impl FnOnce(),
) -> Result<(), MigrationError> {
    T::owner(env).require_auth();

    ensure_is_migrating(env)?;

    custom_migration();
    complete_migration(env);

    UpgradedEvent {
        version: T::version(env),
    }
    .emit(env);

    Ok(())
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

impl Event for UpgradedEvent {
    fn topic() -> impl Topics {
        (symbol_short!("upgraded"),)
    }

    fn data(&self) -> impl IntoVal<Env, Val> {
        (self.version.to_val(),)
    }
}

impl TryFromVal<Env, (Vec<Val>, Val)> for UpgradedEvent {
    type Error = ConversionError;

    fn try_from_val(env: &Env, (topics, data): &(Vec<Val>, Val)) -> Result<Self, Self::Error> {
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
    use crate::shared_interfaces::{OwnershipClient, UpgradableClient, UpgradedEvent};
    use crate::{assert_invoke_auth_err, assert_invoke_auth_ok, shared_interfaces, testdata};
    use std::format;

    use crate::events::match_last_emitted_event;
    use crate::testdata::contract::ContractClient;
    use soroban_sdk::testutils::{Address as _, MockAuth, MockAuthInvoke};
    use soroban_sdk::{contracttype, Address, Env, String};

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
    fn owner_fails_if_owner_not_set() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        assert!(OwnershipClient::new(&env, &contract_id)
            .try_owner()
            .is_err());
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
    fn upgrade_fails_if_owner_not_set() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        assert!(UpgradableClient::new(&env, &contract_id)
            .try_upgrade(&hash)
            .is_err());
    }

    #[test]
    fn upgrade_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        assert!(UpgradableClient::new(&env, &contract_id)
            .try_upgrade(&hash)
            .is_err());
    }

    #[test]
    fn upgrade_fails_if_called_by_non_owner() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        let client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_err!(Address::generate(&env), client.try_upgrade(&hash));
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

        let client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));
    }

    #[test]
    fn migrate_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        let upgrade_client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, upgrade_client.try_upgrade(&hash));

        let client = ContractClient::new(&env, &contract_id);
        assert!(client.try_migrate(&()).is_err());
    }

    #[test]
    fn migrate_fails_if_called_by_non_owner() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        let upgrade_client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, upgrade_client.try_upgrade(&hash));

        let client = ContractClient::new(&env, &contract_id);
        assert_invoke_auth_err!(Address::generate(&env), client.try_migrate(&()));
    }

    #[test]
    fn migrate_fails_if_not_called_after_upgrade() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        let client = ContractClient::new(&env, &contract_id);
        assert_invoke_auth_err!(owner, client.try_migrate(&()));
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

        let upgradable_client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, upgradable_client.try_upgrade(&hash));

        let client = ContractClient::new(&env, &contract_id);
        assert!(client.migration_data().is_none());

        assert_invoke_auth_ok!(owner, client.try_migrate(&()));

        assert_eq!(
            client.migration_data(),
            Some(String::from_str(&env, "migrated"))
        );

        let event = match_last_emitted_event::<UpgradedEvent>(&env);
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

        let client = ContractClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, client.try_migrate(&()));
    }

    #[test]
    fn migrate_fails_if_called_twice() {
        let env = Env::default();
        let contract_id = env.register(testdata::contract::Contract, ());
        let hash = env.deployer().upload_contract_wasm(WASM);

        let owner = Address::generate(&env);
        env.as_contract(&contract_id, || {
            shared_interfaces::set_owner(&env, &owner);
        });

        let upgradable_client = UpgradableClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, upgradable_client.try_upgrade(&hash));

        let client = ContractClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, client.try_migrate(&()));

        assert_invoke_auth_err!(owner, client.try_migrate(&()));
    }

    #[contracttype]
    enum DataKey2 {
        Migrating,
    }
}
