use crate::ensure;
use crate::ownership::OwnershipInterface;
use soroban_sdk::{contractclient, contracttype, BytesN, Env, String, Symbol};

#[contractclient(name = "UpgradeableClient")]
pub trait UpgradeableInterface: OwnershipInterface {
    /// Returns the current version of the contract.
    fn version(env: &Env) -> String;

    /// Upgrades the contract to a new WASM hash.
    /// This function checks that the caller can authenticate as the owner of the contract,
    /// then upgrades the contract to a new WASM hash and prepares it for migration.
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>)
    where
        Self: Sized,
    {
        default_upgrade_impl::<Self>(env, new_wasm_hash);
    }
}

pub fn default_upgrade_impl<T: OwnershipInterface>(env: &Env, new_wasm_hash: BytesN<32>) {
    T::owner(env).require_auth();

    env.deployer().update_current_contract_wasm(new_wasm_hash);
    start_migration(env);
}

/// This function checks that the caller can authenticate as the owner of the contract,
/// then runs the custom_migration and finalizes the migration.
/// An event is emitted when the migration, and with it the overall upgrade, is complete.
/// Migration can only be run once, after the standardized_upgrade function has been called.
pub fn standardized_migrate<T: UpgradeableInterface>(
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
    env.storage().instance().set(&DataKey::Migrating, &());
}

fn ensure_is_migrating(env: &Env) -> Result<(), MigrationError> {
    ensure!(
        env.storage().instance().has(&DataKey::Migrating),
        MigrationError::NotAllowed
    );

    Ok(())
}
fn complete_migration(env: &Env) {
    env.storage().instance().remove(&DataKey::Migrating);
}

#[contracttype]
pub enum DataKey {
    Owner,
    Migrating,
}

pub enum MigrationError {
    NotAllowed,
}
