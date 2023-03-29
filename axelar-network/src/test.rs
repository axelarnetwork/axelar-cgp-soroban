#![cfg(test)]

use core::num;

use super::*;
extern crate std;
use crate::{gateway::*, GatewayClient};
use soroban_sdk::{testutils::{Events, Address as _}, bytes, bytesn, vec, Env, IntoVal, BytesN, Bytes, Address, Symbol, Vec,
xdr::{self, FromXdr, ToXdr}, unwrap::UnwrapOptimized};

use rand::rngs::OsRng;
use ed25519_dalek::SigningKey;
use ed25519_dalek::{Signature, Signer, VerifyingKey, Verifier};


#[test]
fn approve_contract_cal() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // Data for Contract Approve
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
        commands: vec![&env, bytes![&env, 0x617070726f7665436f6e747261637443616c6c]], // approveContractCall converted into Bytes, and then sha256 hashed.
        params: vec![&env, params_approve.clone().to_xdr(&env)]
    };

    let THRESHOLD: u128 = 10;
    let WEIGHT: u128 = 1;
    let NUM_OPS: u32 = 10;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), NUM_OPS, THRESHOLD, WEIGHT);

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };
    
    // Initalize
    let initialize_ops: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &initialize_ops.clone().to_xdr(&env));

    // test Approve Contract Call
    let test = input.to_xdr(&env);
    client.execute(&test);


    
    let event0: Operatorship = initialize_ops;
    let event1: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain: params_approve.src_chain, src_addr: params_approve.src_add, src_tx: params_approve.src_tx_ha, src_event: params_approve.src_evnt};
    let event2: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap().unwrap() };
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
        ]
    );


}

#[test]
fn call_contract() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

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

    let event: ContractCall = ContractCall {
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
                (
                    user, 
                    env.crypto().sha256(&payload),
                ).into_val(&env),
                event.into_val(&env)
            )
        ]
    );
}

#[test]
fn transfer_operatorship() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    let new_operators: Operatorship = Operatorship { 
        new_ops: vec![&env, bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000001), bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000002)],
        new_wghts: vec![&env, 1, 1],
        new_thres: 2
    };


    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970]], // approveContractCall converted into Bytes,
        params: vec![&env, new_operators.clone().to_xdr(&env)]
    };

    let THRESHOLD: u128 = 3;
    let WEIGHT: u128 = 1;
    let NUM_OPS: u32 = 3;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), NUM_OPS, THRESHOLD, WEIGHT);

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };

    // Initalize with 3 random operators
    let params_operator: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
    

    // Transfer operatorship to 2 new operators in the variable new_operators
    let test = input.to_xdr(&env);
    client.execute(&test);


    let initialize_ops: Operatorship = params_operator;
    let new_ops: Operatorship = new_operators;
    let success: ExecutedEvent = ExecutedEvent { command_id: data.commandids.get(0).unwrap().unwrap() };
    assert_eq!(
        env.events().all(),
        vec![
            &env,
            (
                contract_id.clone(),
                ().into_val(&env),
                initialize_ops.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                new_ops.into_val(&env)
            ),
            (
                contract_id.clone(),
                ().into_val(&env),
                success.into_val(&env)
            ),
        ]
    );
}

#[test]
#[should_panic]
fn no_operators() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));


    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x617070726f7665436f6e747261637443616c6c]],
        params: vec![&env, bytes![&env, 0x0]]
    };

    let THRESHOLD: u128 = 10;
    let WEIGHT: u128 = 1;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), 0, THRESHOLD, WEIGHT);

    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

}

#[test]
#[should_panic]
fn operators_not_alphabetical() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));

    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: vec![&env, bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000002), bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000001)],
        new_wghts: vec![&env, 1, 1],
        new_thres: 1
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

#[test]
#[should_panic]
fn invalid_weights() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));

    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: vec![&env, bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000001), bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000002)],
        new_wghts: vec![&env, 1],
        new_thres: 1
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

}

#[test]
#[should_panic]
fn invalid_threshold() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));

    // Test Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: vec![&env, bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000001), bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000002)],
        new_wghts: vec![&env, 1, 1],
        new_thres: 3
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));
}

#[test]
#[should_panic]
fn duplicate_operators() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));

    let new_operatorship: Operatorship = Operatorship { 
        new_ops: vec![&env, bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000001), bytesn!(&env, 0x000000000000000000000000000000000000000000000000000000000000002)],
        new_wghts: vec![&env, 1, 1],
        new_thres: 2
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d), bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0x7472616e736665724f70657261746f7273686970], bytes![&env, 0x7472616e736665724f70657261746f7273686970]],
        params: vec![&env, new_operatorship.clone().to_xdr(&env), new_operatorship.clone().to_xdr(&env)]
    };

    let THRESHOLD: u128 = 3;
    let WEIGHT: u128 = 1;
    let NUM_OPS: u32 = 3;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), NUM_OPS, THRESHOLD, WEIGHT);

    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };

    let params_operator: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD
    };
    let admin: Address = Address::random(&env);
    
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    // This test case shows that you having more than 1 transfer operatorship command in one Data input will ignore the second.
    // It also tests the Duplicate Operators error on the second execute() call.
    let test = input.to_xdr(&env);
    client.execute(&test);
    client.execute(&test);

}

#[test]
#[should_panic]
fn low_signature_weight() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
    // approveContractCall converted into Bytes, and then sha256 hashed.
    let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));


    // Data for Contract Approve
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

    let THRESHOLD: u128 = 100;
    let WEIGHT: u128 = 1;
    let NUM_OPS: u32 = 3;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), NUM_OPS, THRESHOLD, WEIGHT);


    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };
    
    // Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: 3
    };
    let admin: Address = Address::random(&env);
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    // test Execute & Approve Contract Call
    let test = input.to_xdr(&env);
    client.execute(&test);

}

#[test]
#[should_panic]
fn invalid_commands() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Gateway);
    let client = GatewayClient::new(&env, &contract_id);

    // transferOperatorship converted into Bytes, and then sha256 hashed.
    let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
    // approveContractCall converted into Bytes, and then sha256 hashed.
    let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));


    // Data for Contract Approve
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
        commandids: vec![&env, bytesn!(&env, 0x0000000000000000000000000000000000000000000000000000000000000000)],
        commands: vec![&env, bytes![&env, 0x0000000000000000000000000000000000000000000000000000000000000000], bytes![&env, 0x0000000000000000000000000000000000000000000000000000000000000000]],
        params: vec![&env, params_approve.clone().to_xdr(&env)]
    };

    let THRESHOLD: u128 = 3;
    let WEIGHT: u128 = 1;
    let NUM_OPS: u32 = 3;
    let proof: Validate = generate_test_proof(env.clone(), data.clone(), NUM_OPS, THRESHOLD, WEIGHT);


    let input: Input = Input {
        data: data.clone(),
        proof: proof.clone().to_xdr(&env)
    };
    
    // Initalize
    let params_operator: Operatorship = Operatorship { 
        new_ops: proof.operators.clone(),
        new_wghts: proof.weights.clone(),
        new_thres: THRESHOLD
    };
    let admin: Address = Address::random(&env);
    client.initialize(&admin, &params_operator.clone().to_xdr(&env));

    let test = input.to_xdr(&env);
    client.execute(&test);

}

fn generate_test_proof(env: Env, data: Data, num_ops: u32, threshold: u128, weight: u128) -> Validate {

    let mut operators: Vec<BytesN<32>> = Vec::new(&env);
    let mut signatures: Vec<(u32, BytesN<64>)> = Vec::new(&env);
    let mut weights: Vec<u128> = Vec::new(&env);

    for i in 0..num_ops {

        let mut csprng = OsRng{};
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);

        let hash: BytesN<32> = env.crypto().sha256(&data.clone().to_xdr(&env));
        let signed_message_hash: BytesN<32> = to_signed_msg_hsh(env.clone(), hash);
        let message: &[u8] = &signed_message_hash.to_array();

        let signature: Signature = signing_key.sign(message);
        let signature_bytes: BytesN<64> = BytesN::from_array(&env, &signature.to_bytes());
        let verifying_key: VerifyingKey = signing_key.verifying_key();
        let verifying_key_bytes: BytesN<32> = BytesN::from_array(&env, &verifying_key.to_bytes());

        env.crypto().ed25519_verify(
            &verifying_key_bytes,
            &signed_message_hash.into(), 
            &signature_bytes
        );

        for j in 0..operators.len() {
            if verifying_key_bytes.clone() < operators.get(j).unwrap().unwrap() {
                operators.insert(j, verifying_key_bytes.clone());
                signatures.insert(j, (j, signature_bytes.clone()));
                weights.insert(j, weight);

                // Suppose PK #1 is inserted infront of PK #2.
                // Then the associated signature tuple in signatures for PK #2 has the integer equal to the
                // previous index of PK #2 which now equals the index for PK #1.
                // To fix this, we update the integer in signature's tuple by one, for all signature tuples that
                // had its associated PK moved in the operators vector.
                for k in j+1..signatures.len() {
                    let mut temp = signatures.get(k).unwrap().unwrap();
                    temp.0 += 1;
                    signatures.remove(k);
                    signatures.insert(k, temp);
                }
                break;
            } else if j == operators.len() - 1 {
                // public key is bigger than all keys in operators.
                operators.push_back(verifying_key_bytes.clone());
                signatures.push_back((j + 1, signature_bytes.clone()));
                weights.push_back(weight);
            }
        }

        if operators.is_empty() {
            operators.push_back(verifying_key_bytes);
            signatures.push_back((0, signature_bytes));
            weights.push_back(weight);

        }
    }

    let proof: Validate = Validate {
        operators: operators.clone(),
        weights: weights.clone(),
        threshold, // uint256
        signatures: signatures.clone()
    };

    proof
}