#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Events, bytes, vec, Env, IntoVal};
extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    let params = ContractPayload {
        src_chain: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_add: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        contract: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        payload_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_tx_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_evnt: 1 // source event index // do u256 instead?
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d]],
        params: vec![&env, params.clone().serialize(&env)]
    };

    let input: Input = Input {
        data: data.clone(),
        proof: bytes![&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d]
    };

    let test = input.serialize(&env);
    client.execute(&test);
    let event: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain: params.src_chain, src_addr: params.src_add, src_tx: params.src_tx_ha, src_event: params.src_evnt};
    let event2: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap().unwrap() };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                (
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                ).into_val(&env),
                event.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                event2.into_val(&env)
            )
        ]
    );


}