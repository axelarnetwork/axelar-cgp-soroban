#![no_std]
use soroban_sdk::{contractimpl, Bytes, BytesN, Env, Symbol, Vec, Address, map};
use ethabi::{encode, decode};
use sha3::Keccak256;

const SELECTOR_TRANSFER_OPERATORSHIP: u8 = 0;
const SELECTOR_APPROVE_CONTRACT_CALL: u8 = 1;

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn execute(
        chainId: u128,
        commandIds: Vec<BytesN<32>>,
        commands: Vec<u8>,
        params: Vec<Bytes>
    ) {
        let commandsLength: i128 = commandIds.len();

        if (commandsLength != commands.len() || commandsLength != params.len()) {
            // implement
        }

        for i in 0..commandsLength {
            let commandId: BytesN<32> = commandIds[i];
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