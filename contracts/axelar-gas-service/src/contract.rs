use soroban_sdk::token::TokenClient;
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, U256};

use crate::interface::AxelarGasServiceInterface;
use crate::event;
use crate::storage_types::DataKey;

#[contract]
pub struct AxelarGasService;

#[contractimpl]
impl AxelarGasService {
    
    pub fn initialize(env: Env) {
        if env.storage().instance().get(&DataKey::Initialized).unwrap_or(false) {
            panic!("Already initialized");
        }

        env.storage().instance().set(&DataKey::Initialized, &true);
    }
}

#[contractimpl]
impl AxelarGasServiceInterface for AxelarGasService {
    
    fn pay_native_gas_for_contract_call(env: Env, sender: Address, destination_chain: String, destination_address: String, payload: Bytes, refund_address: Address) {
        event::native_gas_paid_for_contract_call(&env, sender, destination_chain, destination_address, payload, refund_address)
    }
    
    fn collect_fees() {
    }

    
    fn refund(env: Env, tx_hash: BytesN<32>, log_index: U256, receiver: Address, token_addr: Address, amount: i128) {
        
        //TODOs
        // 1. need modifier function to ensure only callable by collector
        // 2. confirm whether we want to generalize for non-native tokens
        
        event::refunded(&env, tx_hash, log_index, receiver, token_addr, amount);
                
        let token = TokenClient::new(&env, &token_addr);
        
        // 3. send tokens to receiver. test this
        token.transfer(&env.current_contract_address(), &receiver, &amount)
        
    }

    
}