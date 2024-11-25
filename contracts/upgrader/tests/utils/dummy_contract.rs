use axelar_soroban_std::ownership::OwnershipInterface;
use axelar_soroban_std::upgrade;
use axelar_soroban_std::upgrade::UpgradeableInterface;
use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};

/// A simple contract to test the upgrader
#[contract]
pub struct DummyContract;

/// Dummy contract logic before upgrade
#[contractimpl]
impl UpgradeableInterface for DummyContract {
    fn version(env: &Env) -> soroban_sdk::String {
        soroban_sdk::String::from_str(env, "0.1.0")
    }

    // override the default upgrade function with a simpler one
    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        Self::owner(env).require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

#[contractimpl]
impl OwnershipInterface for DummyContract {}

#[contractimpl]
impl DummyContract {
    pub fn __constructor(env: Env, owner: Address) {
        env.storage()
            .instance()
            .set(&upgrade::DataKey::Owner, &owner)
    }
}

#[contracttype]
pub enum DataKey {
    Data,
}

#[contracterror]
pub enum ContractError {
    SomeFailure = 1,
}

// Dummy contract logic after upgrade is available as testdata/dummy.wasm
//
// #[contractimpl]
// impl UpgradeableInterface for DummyContract {
//     fn version(env: Env) -> String {
//         String::from_str(&env, "0.2.0")
//     }
//
//     fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
//         Self::owner(&env).require_auth();
//
//         env.deployer().update_current_contract_wasm(new_wasm_hash);
//     }
// }
//
// #[contractimpl]
// impl OwnershipInterface for DummyContract {
//     // boilerplate necessary for the contractimpl macro to include function in the generated client
//     fn owner(env: &Env) -> Address {
//         ownership::default_owner_impl(env)
//     }
// }
//
// #[contractimpl]
// impl DummyContract {
//     pub fn migrate(env: Env, migration_data: String) -> Result<(), ContractError> {
//         Self::owner(&env).require_auth();
//
//         env.storage()
//             .instance()
//             .set(&DataKey::Data, &migration_data);
//
//         Ok(())
//     }
// }
