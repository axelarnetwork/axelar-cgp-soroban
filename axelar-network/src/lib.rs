#![no_std]
use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Symbol, vec, Address, map, Vec, crypto, bytesn};
//use alloc::vec::Vec;

extern crate alloc;


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub chain_id: u64,
    pub commandids: Vec<Bytes>,
    pub commands: Vec<Bytes>, //instead of String
    pub params: Vec<Bytes>
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    pub data: Data,
    pub proof: Bytes
}

pub struct Contract;
mod utils;
mod test;
#[contractimpl]
impl Contract {

    pub fn execute (
        env: Env,
        input: Input
    ) {
        
        // dummy values below
        let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
        let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
        
        let data: Data = input.data;
        let proof: Bytes = input.proof;

        let mut allowOperatorshipTransfer: bool = false; // implement

        let chain_id: u64 = data.chain_id;
        let command_ids: Vec<Bytes> = data.commandids;
        let commands: Vec<Bytes> = data.commands;
        let params: Vec<Bytes> = data.params;

        let commands_length: u32 = command_ids.len();


        // if (commandsLength != commands.len() || commandsLength != params.len()) {
        //     // implement
        // }
        for i in 0..commands_length {
            let command_id: Bytes = command_ids.get(i).unwrap().unwrap();

            let command_selector: BytesN<4>;
            let command_hash: BytesN<32> = env.crypto().sha256(&command_id);

            if command_hash == SELECTOR_TRANSFER_OPERATORSHIP {
                if (!allowOperatorshipTransfer) {
                    continue;
                }
                allowOperatorshipTransfer = false;
                // implement
            }
            else if command_hash == SELECTOR_APPROVE_CONTRACT_CALL {
                // implement
            }

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