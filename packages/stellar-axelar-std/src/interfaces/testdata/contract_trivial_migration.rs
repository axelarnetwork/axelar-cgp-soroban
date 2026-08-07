use core::convert::Infallible;

use stellar_axelar_std::{
    contract, contracterror, contracttype, vec, Address, BytesN, Env, String, Vec,
};
use stellar_axelar_std_derive::contractimpl;

use crate as stellar_axelar_std;
use crate::interfaces::{
    operatable, ownable, upgradable, CustomMigratableInterface, MigratableInterface,
    OperatableInterface, OwnableInterface, UpgradableInterface,
};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn __constructor(env: Env, owner: Option<Address>, operator: Option<Address>) {
        if let Some(owner) = owner {
            ownable::set_owner(&env, &owner);
        }

        if let Some(operator) = operator {
            operatable::set_operator(&env, &operator);
        }
    }

    #[allow_during_migration]
    pub fn migration_data(env: &Env) -> Option<String> {
        env.storage().instance().get(&DataKey::Data)
    }
}

// this should normally not be implemented manually, but is done here for testing purposes
#[contractimpl]
impl MigratableInterface for Contract {
    type Error = TrivialContractError;

    #[allow_during_migration]
    fn migrate(env: &Env, migration_data: ()) -> Result<(), TrivialContractError> {
        upgradable::migrate::<Self>(env, migration_data)
            .map_err(|_| TrivialContractError::SomeFailure)
    }
}

impl CustomMigratableInterface for Contract {
    type MigrationData = ();
    type Error = Infallible;

    fn __migrate(env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error> {
        env.storage()
            .instance()
            .set(&DataKey::Data, &String::from_str(env, "migrated"));

        Ok(())
    }
}

#[contractimpl]
impl OwnableInterface for Contract {
    #[allow_during_migration]
    fn owner(env: &Env) -> Address {
        ownable::owner(env)
    }

    fn transfer_ownership(env: &Env, new_owner: Address) {
        ownable::transfer_ownership::<Self>(env, new_owner);
    }
}

#[contractimpl]
impl OperatableInterface for Contract {
    #[allow_during_migration]
    fn operator(env: &Env) -> Address {
        operatable::operator(env)
    }

    fn transfer_operatorship(env: &Env, new_operator: Address) {
        operatable::transfer_operatorship::<Self>(env, new_operator);
    }
}

// this should normally not be implemented manually, but is done here for testing purposes
#[contractimpl]
impl UpgradableInterface for Contract {
    #[allow_during_migration]
    fn version(env: &Env) -> String {
        String::from_str(env, "0.1.0")
    }

    #[allow_during_migration]
    fn required_auths(env: &Env) -> Vec<Address> {
        vec![env, Self::owner(env)]
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
#[derive(Debug)]
pub enum TrivialContractError {
    SomeFailure = 1,
}

mod test {
    use stellar_axelar_std::{contracttype, Address, Env};

    use super::{Contract, DataKey};
    use crate as stellar_axelar_std;

    #[test]
    fn contracttype_enum_name_is_irrelevant_for_key_collision() {
        let env = Env::default();
        let contract_id = env.register(Contract, (None::<Address>, None::<Address>));

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
