#![no_std]
use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Symbol, vec, Address, map};
use ethabi::{encode, decode, ParamType, Token};
use utils::{clean_payload};
use sha3::Keccak256;
use alloc::vec::Vec;

extern crate alloc;


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub chain_id: u64,
    pub commandids: Bytes,
    pub commands: Bytes, //instead of String
    pub params: Bytes
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    pub data: Data,
    pub proof: Bytes
}

const SELECTOR_TRANSFER_OPERATORSHIP: u8 = 0;
const SELECTOR_APPROVE_CONTRACT_CALL: u8 = 1;

pub struct Contract;
mod utils;
mod test;
#[contractimpl]
impl Contract {

    pub fn execute (
        env: Env,
        input: Input
    ) {

        //let payload: Vec<u8> = clean_payload(input);
        // Assume that input is a cleaned payload. That is, a payload without the 0x at the start.
        // let tokens: Vec<Token>  = decode(&alloc::vec![ParamType::Bytes, ParamType::Bytes], &payload).unwrap();
        // // current issue: the type of payload doesn't match up with the parameter type for abi_decode.
        
        let data: Data = input.data;
        let proof: Bytes = input.proof;

        let chain_id: u64 = data.chain_id;
        let command_ids: Bytes = data.commandids;
        let commands: Bytes = data.commands;
        let params: Bytes = data.params;

        let commands_length: u32 = command_ids.len();

        // if (commandsLength != commands.len() || commandsLength != params.len()) {
        //     // implement
        // }

        for i in 0..commands_length {
            //let commandId: u8 = command_ids.get(i);

            //let commandSelector: BytesN<4>;

            // implement
        }



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