use soroban_sdk::{contract, contractimpl, Address, testutils::Address as _, Env};

mod ownable {
    use axelar_soroban_std_derive::ownable;
    use axelar_soroban_std::interfaces::{OwnableClient, OwnableInterface};

    use super::*;

    #[ownable]
    #[contract]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            axelar_soroban_std::interfaces::set_owner(env, &owner);
        }
    }

    #[test]
    fn ownable_contract() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner.clone(),));
        let client = OwnableClient::new(&env, &contract_id);
        let contract_owner = client.owner();
        assert_eq!(owner, contract_owner);
    }
}


mod upgradable {
    use axelar_soroban_std_derive::upgradable;
    use axelar_soroban_std::interfaces::{UpgradableClient, OwnableInterface, UpgradableInterface};

    use super::*;

    #[upgradable]
    #[contract]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            axelar_soroban_std::interfaces::set_owner(env, &owner);
        }
    }

    #[test]
    fn upgradable_contract() {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner,));
        let client = UpgradableClient::new(&env, &contract_id);
        let contract_version = client.version();
        assert_eq!(contract_version.to_string(), env!("CARGO_PKG_VERSION"));
    }
}
