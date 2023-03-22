#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Events, Address as _}, bytes, vec, Env, IntoVal};
extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
    // approveContractCall converted into Bytes, and then sha256 hashed.
    let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));

    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        //NEXT: use public key instead of array containing 1 for new_ops
        new_ops: vec![&env, bytesn!(&env, [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])], 
        new_wghts: vec![&env, 1], 
        new_thres: 1
    };
    let admin: Address = Address::random(&env);

    client.initialize(&admin, &vec![&env, params_operator.clone().serialize(&env)]);

    // Test Contract Approve
    let params_approve = ContractPayload {
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
        commands: vec![&env, bytes![&env, 0x617070726f7665436f6e747261637443616c6c]],
        params: vec![&env, params_approve.clone().serialize(&env)]
    };

    let proof: Validate = Validate {
        operators: vec![&env, bytesn!(&env, [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])],
        weights: vec![&env, 1], // uint256
        threshold: 1, // uint256
        signatures: vec![&env, (0, bytesn!(&env, [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]))]
    };

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().serialize(&env)
    };

    let test = input.serialize(&env);
    client.execute(&test);


    // Test Call Contract
    let user: Address = Address::random(&env);
    let ETHEREUM_ID: Bytes = bytes!(&env, 0x0);
    let JUNKYARD: Bytes = bytes!(&env, 0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59);
    let payload: Bytes = bytes!(&env, 0x000000000000000000000000da2982fa68c3787af86475824eeb07702c4c449f00000000000000000000000000000000000000000000000000000000000003be0000000000000000000000004efe356bedecc817cb89b4e9b796db8bc188dc59);
    client.call_con(
        &user, 
        &ETHEREUM_ID, 
        &JUNKYARD, 
        &payload
    );


    let event0: Operatorship =  params_operator;
    let event1: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain: params_approve.src_chain, src_addr: params_approve.src_add, src_tx: params_approve.src_tx_ha, src_event: params_approve.src_evnt};
    let event2: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap().unwrap() };
    let event3: ContractCall = ContractCall {
        prefix: symbol!("ContractC"),
        dest_chain: ETHEREUM_ID,
        dest_addr: JUNKYARD,
        payload: payload.clone()
    };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                ().into_val(&env),
                event0.into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
                ).into_val(&env),
                event1.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                event2.into_val(&env)
            ),
            (
                contract_id.clone(),
                (
                    user, 
                    env.crypto().sha256(&payload),
                ).into_val(&env),
                event3.into_val(&env)
            )
        ]
    );


}
