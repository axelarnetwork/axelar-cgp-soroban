use axelar_soroban_std::shared_interfaces;
use axelar_soroban_std::shared_interfaces::OwnershipInterface;
use axelar_soroban_std::shared_interfaces::UpgradeableInterface;
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

    fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
        Self::owner(env).require_auth();

        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

#[contractimpl]
impl OwnershipInterface for DummyContract {
    fn owner(env: &Env) -> Address {
        shared_interfaces::owner(env)
    }
}

#[contractimpl]
impl DummyContract {
    pub fn __constructor(env: Env, owner: Address) {
        shared_interfaces::set_owner(&env, &owner);
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
//
// /// A simple contract to test the upgrader
// #[contract]
// pub struct DummyContract;
//
// /// Dummy contract logic before upgrade
// #[contractimpl]
// impl UpgradeableInterface for DummyContract {
//     fn version(env: &Env) -> soroban_sdk::String {
//         soroban_sdk::String::from_str(env, "0.2.0")
//     }
//
//     fn upgrade(env: &Env, new_wasm_hash: BytesN<32>) {
//         Self::owner(env).require_auth();
//
//         env.deployer().update_current_contract_wasm(new_wasm_hash);
//     }
// }
//
// #[contractimpl]
// impl OwnershipInterface for DummyContract {
//     fn owner(env: &Env) -> Address {
//         shared_interfaces::owner(env)
//     }
// }
//
// #[contractimpl]
// impl DummyContract {
//     pub fn __constructor(env: Env, owner: Address) {
//         shared_interfaces::set_owner(&env, &owner);
//     }
//
//     pub fn migrate(env: Env, migration_data: soroban_sdk::String) -> Result<(), ContractError> {
//         Self::owner(&env).require_auth();
//         env.storage()
//             .instance()
//             .set(&DataKey::Data, &migration_data);
//         Ok(())
//     }
// }
