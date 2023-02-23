#![no_std]
use soroban_sdk::{contractimpl, bytes, Bytes, BytesN, Env, Symbol, vec, Vec, Address, map};
use ethabi::{encode, decode, ParamType, Token};
use utils::{clean_payload};
use sha3::Keccak256;

extern crate alloc;

const SELECTOR_TRANSFER_OPERATORSHIP: u8 = 0;
const SELECTOR_APPROVE_CONTRACT_CALL: u8 = 1;

pub struct Contract;
mod utils;
mod test;
#[contractimpl]
impl Contract {

    pub fn execute (
        env: Env,
        input: Bytes
    ) -> Vec<Token> {

        let payload = clean_payload(input);
        // Assume that input is a cleaned payload. That is, a payload without the 0x at the start.
        let tokens = decode(&alloc::vec![ParamType::Bytes, ParamType::Bytes], &payload).unwrap();
        // current issue: the type of payload doesn't match up with the parameter type for abi_decode.
        
        //let data = tokens[0].clone().into_bytes().unwrap();
        //let proof = tokens[1].clone().into_bytes().unwrap();
        tokens
        //*proof.last().unwrap() as u32
        
    //     let expected_output_types = std::vec![
    //         ParamType::Uint(256),
    //         ParamType::Array(Box::new(ParamType::FixedBytes(32))),
    //         ParamType::Array(Box::new(ParamType::String)),
    //         ParamType::Array(Box::new(ParamType::Bytes))
    //     ];

    //     let data_tokens = abi_decode(&data, &expected_output_types).unwrap();
    //     let command_ids = data_tokens[1]
    //     .clone()
    //     .into_array()
    //     .unwrap()
    //     .into_iter()
    //     .map(|token| token.into_fixed_bytes().unwrap())
    //     .collect::<Vec<_>>();

    // // let commands = data_tokens[2]
    // //     .clone()
    // //     .into_array()
    // //     .unwrap()
    // //     .into_iter()
    // //     .map(|token| into_bytes(token).unwrap())
    // //     .collect::<Vec<_>>();

    // let params = data_tokens[3]
    //     .clone()
    //     .into_array()
    //     .unwrap()
    //     .into_iter()
    //     .map(|token| token.into_bytes().unwrap())
    //     .collect::<Vec<_>>();

        // let commandsLength: u32 = commandIds.len();

        // if (commandsLength != commands.len() || commandsLength != params.len()) {
        //     // implement
        // }

        // for i in 0..commandsLength {
        //     let commandId: BytesN<32> = commandIds.get(i);
        //     // implement
        // }



    }

    pub fn approve( // approveContractCall
        env: Env,
        commandId: BytesN<32>,
        sourceChain: Symbol,
        sourceAddress: Address,
        contractAddress: Address,
        payloadHash: BytesN<32>,
        sourceTxHash: BytesN<32>,
        sourceEventIndex: u128
    ) {
        // implement

        // let data = map![&env, (1, sourceChain), (2, sourceAddress), (3, contractAddress), (4, sourceTxHash), (5, sourceEventIndex)];
        // env.events().publish((commandId, contractAddress, payloadHash), data);
    }

    pub fn transferOp( // transferOperatorship
        env: Env,
        newOperatorsData: Bytes
    ) {
        // implement
        // env.events().publish(1, newOperatorsData);
    }

}