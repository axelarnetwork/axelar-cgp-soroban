#![cfg(test)]
extern crate std;

use crate::{contract::{self, AxelarGateway}, AxelarGatewayClient};
use soroban_sdk::{bytes, vec, symbol_short, testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, Events}, Address, Bytes, BytesN, Env, IntoVal, String, Symbol};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

#[test]
fn call_contract() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGateway);
    let client = AxelarGatewayClient::new(&env, &contract_id);

    let user: Address = Address::generate(&env);
    let destination_chain = String::from_str(&env, DESTINATION_CHAIN);
    let destination_address = String::from_str(&env, DESTINATION_ADDRESS);
    let payload = bytes!(&env, 0x1234);

    client.call_contract(&user, &destination_chain, &destination_address, &payload);

    assert_eq!(
        env.auths(),
        std::vec![(
            user.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    client.address.clone(),
                    Symbol::new(&env, "call_contract"),
                    (&user, destination_chain.clone(), destination_address.clone(), payload.clone()).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    let events = env.events().all();

    assert_eq!(events.len(), 1);
    assert_eq!(
        events,
        vec![
            &env,
            (
                contract_id.clone(),
                (
                    symbol_short!("called"),
                    user,
                    env.crypto().keccak256(&payload),
                ).into_val(&env),
                (destination_chain, destination_address, payload).into_val(&env)
            )
        ]);
}
