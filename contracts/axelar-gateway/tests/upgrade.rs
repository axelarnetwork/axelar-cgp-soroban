#![cfg(test)]

use axelar_gateway::testutils::setup_gateway;

use soroban_sdk::{Env, String};

// For reproducibility:
// 1. Update the package version in Cargo.toml to reflect new changes.
// 2. Execute `stellar contract build` to build the contract.
// 3. Run `optimize.sh` to produce the optimized WASM file.
// 4. Rename the generated file and move it under the `testdata` directory.

mod old_contract {
    soroban_sdk::contractimport!(
        file = "./tests/testdata/axelar_gateway_old.wasm"
    );
}

mod new_contract {
    soroban_sdk::contractimport!(
        file = "./tests/testdata/axelar_gateway_new.wasm"
    );
}

const OLD_CONTRACT_VERSION: &str = "0.1.0";
const NEW_CONTRACT_VERSION: &str = "0.1.1";

#[test]
fn upgrade() {
    let env = Env::default();
    env.mock_all_auths();

    let (_signers, client) = setup_gateway(&env, 0, 5);

    assert_eq!(
        String::from_str(&env, OLD_CONTRACT_VERSION),
        client.version()
    );

    let new_wasm_hash = env.deployer().upload_contract_wasm(new_contract::WASM);

    client.upgrade(&new_wasm_hash);

    assert_eq!(
        String::from_str(&env, NEW_CONTRACT_VERSION),
        client.version()
    );
}
