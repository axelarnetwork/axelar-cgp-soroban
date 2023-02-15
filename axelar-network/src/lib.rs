#![no_std]

use soroban_sdk::{contractimpl, Bytes, BytesN, Env, Symbol, Vec, Address};

pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn approve( // approveContractCall
        env: Env,
        commandId: BytesN<32>,
        sourceChain: Symbol,
        sourceAddress: Address,
        destinationAddress: Address,
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