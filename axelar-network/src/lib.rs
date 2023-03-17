#![no_std]
use auth::{to_signed_msg_hsh, validate_proof, transfer_op};
use soroban_sdk::{contractimpl, contracttype, bytes, Bytes, BytesN, Env, Symbol, symbol, vec, Address, Map, map, Vec, crypto, bytesn,
    serde::{Deserialize, Serialize}, xdr::Uint256
};
//use alloc::vec::Vec;
use stellar_xdr;

mod auth;


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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Operatorship {
    pub new_ops: Vec<BytesN<32>>, // new_operators
    pub new_wghts: Vec<u128>, // new_weights change to Uint256
    pub new_thres: u128 // new_threshold change to Uint256
}

pub struct Contract;
//  {
//     // Auth Weighted
//     crnt_epoch: u64, //current_epoch
//     hash_epoch: Map<u64, BytesN<32>>, // hash_for_epoch
//     epoch_hash: Map<BytesN<32>, u64> // epoch_for_hash

// }

//mod utils;
mod test;
#[contractimpl]
impl Contract {
    pub fn execute (
        env: Env,
        input: Bytes
    ) {
        
        // transferOperatorship converted into Bytes, and then sha256 hashed.
        let SELECTOR_TRANSFER_OPERATORSHIP: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x7472616e736665724f70657261746f7273686970));
        // approveContractCall converted into Bytes, and then sha256 hashed.
        let SELECTOR_APPROVE_CONTRACT_CALL: BytesN<32> = env.crypto().sha256(&bytes!(&env, 0x617070726f7665436f6e747261637443616c6c));
        
        let decoded: Input = Input::deserialize(&env, &input).unwrap();
        let data: Data = decoded.data;
        let proof: Bytes = decoded.proof;
        let hash: BytesN<32> = env.crypto().sha256(&data.clone().serialize(&env));
        let signed_message_hash: BytesN<32> = to_signed_msg_hsh(env.clone(), hash);
        let mut allowOperatorshipTransfer: bool = validate_proof(env.clone(), signed_message_hash, proof.clone());
        
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
                Self::_setCommandExecuted(env.clone(), command_id.clone(), true);
                success = transfer_op(env.clone(), params.get(i).unwrap().unwrap());
            }
            else if command_hash == SELECTOR_APPROVE_CONTRACT_CALL { 
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