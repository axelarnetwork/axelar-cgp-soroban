#![cfg(test)]

use super::*;
extern crate std;
use crate::{gateway::*, GatewayClient};
use soroban_sdk::{testutils::{Events, Address as _}, bytes, bytesn, vec, Env, IntoVal, BytesN, Bytes, Address, Symbol,
xdr::{self, FromXdr, ToXdr}};

use rand::rngs::OsRng;
use ed25519_dalek::SigningKey;
use ed25519_dalek::{Signature, Signer, VerifyingKey, Verifier};


#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
    // approveContractCall converted into Bytes, and then sha256 hashed.
    let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));


    // sign something first and then verify natively
    let mut csprng = OsRng{};
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);


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
        params: vec![&env, params_approve.clone().to_xdr(&env)]
    };

    // for generating public key
    let hash: BytesN<32> = env.crypto().sha256(&data.clone().to_xdr(&env));
    let signed_message_hash: BytesN<32> = to_signed_msg_hsh(env.clone(), hash);
    let message: &[u8] = &signed_message_hash.to_array();
    let signature: Signature = signing_key.sign(message);
    let signature_bytes: BytesN<64> = BytesN::from_array(&env, &signature.to_bytes());
    let verifying_key: VerifyingKey = signing_key.verifying_key();
    let verifying_key_bytes: BytesN<32> = BytesN::from_array(&env, &verifying_key.to_bytes());
    // for generating signature & public key 

    let proof: Validate = Validate {
        operators: vec![&env, verifying_key_bytes.clone()],
        weights: vec![&env, 1], // uint256
        threshold: 1, // uint256
        signatures: vec![&env, (0, signature_bytes)]
    };

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };
    
    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: vec![&env, verifying_key_bytes], 
        new_wghts: vec![&env, 1], 
        new_thres: 1
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &vec![&env, params_operator.clone().to_xdr(&env)]);

    // test Execute & Approve Contract Call
    let test = input.to_xdr(&env);
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
        prefix: Symbol::new(&env, &"ContractCall"),
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
