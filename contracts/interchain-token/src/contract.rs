use soroban_sdk::{contract, contractimpl, Address, Env};

use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct InterchainToken;

#[contractimpl]
impl InterchainToken {
    pub fn __constructor(env: Env, owner: Address) {
        env.storage().instance().set(&DataKey::Owner, &owner);
    }

    pub fn owner(env: &Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Owner)
            .expect("owner not found")
    }

    pub fn transfer_ownership(env: Env, new_owner: Address) {
        let owner: Address = Self::owner(&env);
        owner.require_auth();

        env.storage().instance().set(&DataKey::Owner, &new_owner);

        event::transfer_ownership(&env, owner, new_owner);
    }
}
