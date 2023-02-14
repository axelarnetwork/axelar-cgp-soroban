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
        // use events to emit events.
    }

    pub fn transferOp( // transferOperatorship
        env: Env,
        newOperatorsData: Bytes
    ) {
        // implement
        // use events to emit events.
    }

}