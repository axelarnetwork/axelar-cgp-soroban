use crate::upgrade::DataKey;
use soroban_sdk::{contractclient, Address, Env};

#[contractclient(name = "OwnershipClient")]
pub trait OwnershipInterface {
    fn owner(env: &Env) -> Address {
        default_owner_impl(env)
    }
}

pub fn default_owner_impl(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Owner)
        .expect("owner not found")
}
