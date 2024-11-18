use crate::error::ContractError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Symbol, Val};

#[contract]
pub struct Upgrader;

#[contractimpl]
impl Upgrader {
    pub fn __constructor(_env: Env) {}
    pub fn upgrade(
        env: Env,
        contract_address: Address,
        new_version: String,
        new_wasm_hash: BytesN<32>,
        migration_data: soroban_sdk::Vec<Val>,
    ) -> Result<(), ContractError> {
        assert_new_version_is_different(&env, &contract_address, &new_version)?;

        env.invoke_contract::<()>(
            &contract_address,
            &upgrade_fn(&env),
            soroban_sdk::vec![&env, new_wasm_hash.into()],
        );
        env.invoke_contract::<()>(&contract_address, &migrate_fn(&env), migration_data);

        assert_new_version_matches_expected(&env, &contract_address, &new_version)
    }
}

fn migrate_fn(env: &Env) -> Symbol {
    Symbol::new(&env, "migrate")
}

fn upgrade_fn(env: &Env) -> Symbol {
    Symbol::new(&env, "upgrade")
}

fn version_fn(env: &Env) -> Symbol {
    Symbol::new(&env, "version")
}

fn assert_new_version_is_different(
    env: &Env,
    contract_address: &Address,
    new_version: &String,
) -> Result<(), ContractError> {
    let no_match = current_version(&env, &contract_address) != *new_version;
    no_match.then_ok((), ContractError::SameVersion)
}

fn assert_new_version_matches_expected(
    env: &Env,
    contract_address: &Address,
    new_version: &String,
) -> Result<(), ContractError> {
    let versions_match = current_version(&env, &contract_address) == *new_version;
    versions_match.then_ok((), ContractError::VersionMismatch)
}

fn current_version(env: &Env, contract_address: &Address) -> String {
    env.invoke_contract(
        contract_address,
        &version_fn(env),
        soroban_sdk::vec![env],
    )
}

#[cfg(test)]
mod tests {
    use crate::contract::{Upgrader, UpgraderClient};
    use axelar_gateway::AxelarGateway;
    use soroban_sdk::{BytesN, Env, String};

    const _WASM: &[u8] = include_bytes!("testdata/axelar_gateway.wasm");
    #[test]
    fn test() {
        let env = Env::default();

        let contract_id = env.register(Upgrader, ());
        let gateway_id = env.register(AxelarGateway, ());
        let client = UpgraderClient::new(&env, &contract_id);
        client.upgrade(
            &gateway_id,
            &String::from_str(&env, "1.0"),
            &BytesN::from_array(&env, &[0; 32]),
            &soroban_sdk::vec![&env],
        );
    }
}
