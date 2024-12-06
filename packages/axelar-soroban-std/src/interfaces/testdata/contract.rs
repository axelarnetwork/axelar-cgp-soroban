use crate::interfaces::{upgradable, MigratableInterface, OwnableInterface, UpgradableInterface};
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
        upgradable::migrate::<Self>(env, || Self::run_migration(env, migration_data))
            .map_err(|_| ContractError::SomeFailure)
    }
}

#[contractimpl]
impl OwnableInterface for Contract {
    fn owner(env: &Env) -> Address {
        upgradable::owner(env)
    }
}

#[contractimpl]
impl UpgradableInterface for Contract {
    fn version(env: &Env) -> String {
        String::from_str(env, "0.1.0")
    }

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        upgradable::upgrade::<Self>(env, new_wasm_hash);
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

mod test {
    use soroban_sdk::{contracttype, Env};

    use crate::interfaces::testdata::contract::DataKey;

    use super::Contract;

    #[test]
    fn contracttype_enum_name_is_irrelevant_for_key_collision() {
        let env = Env::default();
        let contract_id = env.register(Contract, ());

        env.as_contract(&contract_id, || {
            assert!(!env.storage().instance().has(&DataKey::Migrating));
            assert!(!env.storage().instance().has(&DataKey2::Migrating));

            env.storage().instance().set(&DataKey::Migrating, &());

            assert!(env.storage().instance().has(&DataKey::Migrating));
            assert!(env.storage().instance().has(&DataKey2::Migrating));
        });
    }

    #[contracttype]
    enum DataKey2 {
        Migrating,
    }
}
