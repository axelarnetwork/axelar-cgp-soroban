#![cfg(test)]
extern crate std;

use crate::contract::AxelarGasService;
use soroban_sdk::{Address, Env};

fn setup_env<'a>() -> (Env, Address, AxelarGasService<>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGasService);
    
    //TODO: why am i getting this compilation error on "new"? "no function or associated item named `new` found for struct `AxelarGasService` in the current scope"
    let client = AxelarGasService::new(&env);

    client.initialize(&contract_id);

    (env, contract_id, client)
}