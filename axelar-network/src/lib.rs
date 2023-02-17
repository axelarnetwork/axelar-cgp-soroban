#![no_std]
use soroban_sdk::{contractimpl, Bytes, BytesN, Env, Symbol, Vec, vec, Address, map};
use ethabi::{encode, decode, ParamType};
use crate::utils::{self, abi_encode, abi_decode, clean_payload};
use sha3::Keccak256;

const SELECTOR_TRANSFER_OPERATORSHIP: u8 = 0;
const SELECTOR_APPROVE_CONTRACT_CALL: u8 = 1;

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn execute(
        env: Env,
        input: Bytes
        // chainId: u128,
        // commandIds: Vec<BytesN<32>>,
        // commands: Vec<u32>,
        // params: Vec<Bytes>
    ) {
        //let payload = clean_payload(input.clone());
        // Assume that input is a cleaned payload. That is, a payload without the 0x at the start.
        let tokens = abi_decode(&payload, &[ParamType::Bytes]);//.unwrap();
        // current issue: the type of payload doesn't match up with the parameter type for abi_decode.

        let data = tokens[0].clone().into_bytes().unwrap();
        let proof = tokens[1].clone().into_bytes().unwrap();

        let commandsLength: u32 = commandIds.len();

        if (commandsLength != commands.len() || commandsLength != params.len()) {
            // implement
        }

        for i in 0..commandsLength {
            let commandId: BytesN<32> = commandIds.get(i);
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

        let data = map![&env, (1, sourceChain), (2, sourceAddress), (3, contractAddress), (4, sourceTxHash), (5, sourceEventIndex)];
        env.events().publish((commandId, contractAddress, payloadHash), data);
    }

    pub fn transferOp( // transferOperatorship
        env: Env,
        newOperatorsData: Bytes
    ) {
        // implement
        env.events().publish(1, newOperatorsData);
    }

}