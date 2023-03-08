#![no_std]
use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Symbol, symbol, vec, Address, map, Vec, crypto, bytesn,
    serde::{Deserialize, Serialize}
};
//use alloc::vec::Vec;
use stellar_xdr;

extern crate alloc;


#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Data {
    pub chain_id: u64,
    pub commandids: Vec<BytesN<32>>,
    pub commands: Vec<Bytes>, //instead of Vec<String>
    pub params: Vec<Bytes>
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Input {
    pub data: Data,
    pub proof: Bytes
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractPayload {
    pub src_chain: Bytes,
    pub src_add: Bytes,
    pub contract: Bytes, // contract address
    pub payload_ha: BytesN<32>, // payload hash
    pub src_tx_ha: BytesN<32>, // source tx hash
    pub src_evnt: u64 // source event index // do u256 instead?
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallApprovedEvent {
    //pub command_id: Bytes,
    pub src_chain: Bytes,
    pub src_addr: Bytes,
    //pub contract: Bytes, // contract address
    //pub payload: Bytes,
    pub src_tx: BytesN<32>, // source tx hash
    pub src_event: u64
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCallApprovedKey {
    pub approved: Symbol,
    pub command_id: BytesN<32>,
    pub src_chain: Bytes,
    pub src_addr: Bytes,
    pub contract: Bytes, // contract address
    pub payload_ha: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCall {
    pub dest_chain: Bytes,
    pub dest_addr: Bytes,
    pub payload: Bytes
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutedEvent {
    pub command_id: BytesN<32>
}


pub struct Contract;
mod utils;
mod test;
#[contractimpl]
impl Contract {

    pub fn execute (
        env: Env,
        input: Bytes
    ) {
        
        // dummy values below
        let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = bytesn!(&env, 0xabcd3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
        let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
        
        let decoded: Input = Input::deserialize(&env, &input).unwrap();
        let data: Data = decoded.data;
        let proof: Bytes = decoded.proof; // implement proof check

        let mut allowOperatorshipTransfer: bool = false; // implement

        let chain_id: u64 = data.chain_id;
        let command_ids: Vec<BytesN<32>> = data.commandids;
        let commands: Vec<Bytes> = data.commands;
        let params: Vec<Bytes> = data.params;

        let commands_length: u32 = command_ids.len();


        // if (commandsLength != commands.len() || commandsLength != params.len()) {
        //     // implement
        // }
        for i in 0..commands_length {
            let command_id: BytesN<32> = command_ids.get(i).unwrap().unwrap();
            let command_hash: BytesN<32> = env.crypto().sha256(&commands.get(i).unwrap().unwrap());
            let mut success: bool = false;

            if command_hash == SELECTOR_TRANSFER_OPERATORSHIP {
                if (!allowOperatorshipTransfer) {
                    continue;
                }
                allowOperatorshipTransfer = false;
                // implement
            }
            else if true { //command_hash == SELECTOR_APPROVE_CONTRACT_CALL { // testing purposes
                Self::_setCommandExecuted(env.clone(), command_id.clone(), true);
                success = Self::approve(env.clone(), params.get(i).unwrap().unwrap(), command_id.clone());
            }

            if (success) {
                let event: ExecutedEvent = ExecutedEvent { command_id: command_id.clone() };
                env.events().publish((), event);
            } else {
                Self::_setCommandExecuted(env.clone(), command_id.clone(), false);
            }
        }



    }

    pub fn approve( // approveContractCall
        env: Env,
        params: Bytes,
        command_id: BytesN<32>
    ) -> bool {
        let decoded: ContractPayload = ContractPayload::deserialize(&env, &params).unwrap();
        let src_chain: Bytes = decoded.src_chain;
        let src_addr: Bytes = decoded.src_add;
        let contract: Bytes = decoded.contract;
        let payload_ha: BytesN<32> = decoded.payload_ha;
        let src_tx: BytesN<32> = decoded.src_tx_ha;
        let src_event: u64 = decoded.src_evnt;
        
        Self::_setContractCallApproved(env.clone(), command_id.clone(), src_chain.clone(), src_addr.clone(), contract.clone(), payload_ha.clone());
        let event: ContractCallApprovedEvent = ContractCallApprovedEvent { src_chain, src_addr, src_tx, src_event};
        env.events().publish((command_id, contract, payload_ha), event);

        true
    }
    
    fn _setContractCallApproved( // how do I make this functio internal / protected
        env: Env,
        commandId: BytesN<32>,
        sourceChain: Bytes,
        sourceAddress: Bytes,
        contractAddress: Bytes, // Address instead of Bytes?
        payloadHash: BytesN<32>
    ) {
        let data: ContractCallApprovedKey = ContractCallApprovedKey{
            approved: symbol!("approved"), 
            command_id: commandId,
            src_chain: sourceChain, 
            src_addr: sourceAddress, 
            contract: contractAddress,
            payload_ha: payloadHash
        };
        let key: BytesN<32> = env.crypto().sha256(&data.serialize(&env));
        env.storage().set(&key, &true);
    }

    fn _setCommandExecuted(
        env: Env,
        command_id: BytesN<32>,
        executed: bool
    ) {
        let key: BytesN<32> = env.crypto().sha256(&command_id.serialize(&env));
        env.storage().set(&key, &executed);
    }

    pub fn transferOp( // transferOperatorship
        env: Env,
        newOperatorsData: Bytes
    ) {
        // implement
        // env.events().publish(1, newOperatorsData);
    }

    pub fn call_con(
        env: Env,
        dest_chain: Bytes,
        dest_addr: Bytes,
        payload: Bytes // payload hash
    ) {
        let data: ContractCall = ContractCall {
            dest_chain,
            dest_addr,
            payload: payload.clone()
        };
        //let sender: Address; // implement

        env.events().publish((env.crypto().sha256(&payload),), data);
    }

}