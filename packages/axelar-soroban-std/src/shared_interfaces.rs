use crate::ensure;
use soroban_sdk::{contractclient, Address, BytesN, Env, String, Symbol};

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

pub fn owner(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&storage::DataKey::SharedInterfaces_Owner)
        .expect("owner must be set during contract construction")
}

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
/// Migration can only be run once, after the standardized_upgrade function has been called.
pub fn migrate<T: UpgradeableInterface>(
    env: &Env,
    custom_migration: impl FnOnce(),
) -> Result<(), MigrationError> {
    T::owner(env).require_auth();

    ensure_is_migrating(env)?;

    custom_migration();
    complete_migration(env);

    emit_event_upgraded(env, &T::version(env));

    Ok(())
}

fn emit_event_upgraded(env: &Env, version: &String) {
    env.events()
        .publish((Symbol::new(env, "upgraded"),), (version.to_val(),));
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
    #![allow(clippy::redundant_pub_crate)] // contract macro generates pub types

    use crate::shared_interfaces;
    use soroban_sdk::testutils::arbitrary::std;
    use soroban_sdk::testutils::storage::Instance;
    use soroban_sdk::{contract, contractimpl, contracttype};
    use std::println;

    #[contract]
    struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: soroban_sdk::Env) {
            env.storage().instance().set(&DataKey::Migrating, &true);
        }
    }

    #[contracttype]
    enum DataKey {
        Migrating,
    }

    #[contracttype]
    enum DataKey2 {
        Migrating,
    }

    #[test]
    fn contract_can_use_imported_contracttype_enum() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register(Contract, ());

        env.as_contract(&contract_id, || {
            println!("{:?}", &env.storage().instance().all());
            assert!(env.storage().instance().has(&DataKey::Migrating));
            assert!(!env
                .storage()
                .instance()
                .has(&shared_interfaces::storage::DataKey::SharedInterfaces_Migrating));

            shared_interfaces::start_migration(&env);
            assert!(env
                .storage()
                .instance()
                .has(&shared_interfaces::storage::DataKey::SharedInterfaces_Migrating));
        });
    }

    #[test]
    fn contracttype_enum_name_is_irrelevant_for_key_collision() {
        let env = soroban_sdk::Env::default();
        let contract_id = env.register(Contract, ());

        env.as_contract(&contract_id, || {
            println!("{:?}", &env.storage().instance().all());
            assert!(env.storage().instance().has(&DataKey::Migrating));
            assert!(env.storage().instance().has(&DataKey2::Migrating));
        });
    }
}
