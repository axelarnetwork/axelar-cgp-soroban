use core::fmt::Debug;

use stellar_axelar_std::{contract, contracterror, Address, Env};
use stellar_axelar_std_derive::{contractimpl, Ownable, Upgradable};

use crate as stellar_axelar_std;
use crate::events::Event;
use crate::interfaces::CustomMigratableInterface;
use crate::IntoEvent;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ContractError {
    MigrationNotAllowed = 1,
    MigrationInProgress = 2,
}

#[contract]
#[derive(Ownable, Upgradable)]
#[migratable(data = ())]
pub struct Contract;

#[derive(Debug, PartialEq, Eq, IntoEvent)]
pub struct MigratedEvent {}

#[contractimpl]
impl Contract {
    pub fn __constructor(env: &Env, owner: Address) {
        crate::interfaces::set_owner(env, &owner);
    }
}

impl CustomMigratableInterface for Contract {
    type MigrationData = ();
    type Error = ContractError;

    fn __migrate(env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error> {
        MigratedEvent {}.emit(env);

        Ok(())
    }
}
