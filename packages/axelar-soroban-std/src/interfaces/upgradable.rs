use crate::ensure;
use crate::events::Event;
#[cfg(any(test, feature = "testutils"))]
use crate::impl_event_testutils;
use crate::interfaces::OwnableInterface;
use core::fmt::Debug;
use soroban_sdk::{
    contractclient, symbol_short, BytesN, Env, FromVal, IntoVal, String, Topics, Val,
};

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
        .set(&storage::DataKey::Interfaces_Migrating, &());
}

fn ensure_is_migrating(env: &Env) -> Result<(), MigrationError> {
    ensure!(
        env.storage()
            .instance()
            .has(&storage::DataKey::Interfaces_Migrating),
        MigrationError::NotAllowed
    );

    Ok(())
}

fn complete_migration(env: &Env) {
    env.storage()
        .instance()
        .remove(&storage::DataKey::Interfaces_Migrating);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradedEvent {
    version: String,
}

impl Event for UpgradedEvent {
    fn topics(&self, _env: &Env) -> impl Topics + Debug {
        (symbol_short!("upgraded"),)
    }

    fn data(&self, _env: &Env) -> impl IntoVal<Env, Val> + Debug {
        (self.version.to_val(),)
    }
}

#[cfg(any(test, feature = "testutils"))]
impl_event_testutils!(UpgradedEvent, (soroban_sdk::Symbol), (String));

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
        Interfaces_Migrating,
    }
}

pub enum MigrationError {
    NotAllowed,
}

#[cfg(test)]
mod test {
    use crate::interfaces::upgradable::UpgradedEvent;
    use crate::{assert_invoke_auth_err, assert_invoke_auth_ok, events};

    use crate::interfaces::testdata::ContractClient;
    use crate::interfaces::{testdata, upgradable};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{Address, BytesN, Env, String};

    const WASM: &[u8] = include_bytes!("testdata/contract.wasm");

    fn prepare_client_and_bytecode(
        env: &Env,
        owner: Option<Address>,
    ) -> (ContractClient, BytesN<32>) {
        let operator = Address::generate(env);
        let contract_id = env.register(testdata::Contract, (owner, operator));
        let hash = env.deployer().upload_contract_wasm(WASM);
        let client = ContractClient::new(env, &contract_id);
        (client, hash)
    }

    #[test]
    fn upgrade_fails_if_owner_not_set() {
        let env = Env::default();
        let (client, hash) = prepare_client_and_bytecode(&env, None);

        assert!(client.try_upgrade(&hash).is_err());
    }

    #[test]
    fn upgrade_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner));

        assert!(client.try_upgrade(&hash).is_err());
    }

    #[test]
    fn upgrade_fails_if_called_by_non_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner));

        assert_invoke_auth_err!(Address::generate(&env), client.try_upgrade(&hash));
    }

    #[test]
    fn upgrade_succeeds_if_owner_is_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));
    }

    #[test]
    fn migrate_fails_if_caller_not_authenticated() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));
        assert!(client.try_migrate(&()).is_err());
    }

    #[test]
    fn migrate_fails_if_called_by_non_owner() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));
        assert_invoke_auth_err!(Address::generate(&env), client.try_migrate(&()));
    }

    #[test]
    fn migrate_fails_if_not_called_after_upgrade() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, _) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_err!(owner, client.try_migrate(&()));
    }

    #[test]
    fn migrate_succeeds_if_owner_is_authenticated_and_called_after_upgrade() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));

        assert!(client.migration_data().is_none());

        assert_invoke_auth_ok!(owner, client.try_migrate(&()));

        assert_eq!(
            client.migration_data(),
            Some(String::from_str(&env, "migrated"))
        );

        goldie::assert!(events::fmt_last_emitted_event::<UpgradedEvent>(&env))
    }

    // Because migration happens on a contract loaded from WASM, code coverage analysis doesn't recognize
    // the migration code as covered. This test repeats the migration test with mocked setup
    #[test]
    fn simulate_migration_for_code_coverage() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(testdata::Contract, (Some(owner.clone()), None::<Address>));

        env.as_contract(&contract_id, || {
            upgradable::start_migration(&env);
        });

        let client = ContractClient::new(&env, &contract_id);
        assert_invoke_auth_ok!(owner, client.try_migrate(&()));
    }

    #[test]
    fn migrate_fails_if_called_twice() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let (client, hash) = prepare_client_and_bytecode(&env, Some(owner.clone()));

        assert_invoke_auth_ok!(owner, client.try_upgrade(&hash));
        assert_invoke_auth_ok!(owner, client.try_migrate(&()));
        assert_invoke_auth_err!(owner, client.try_migrate(&()));
    }
}