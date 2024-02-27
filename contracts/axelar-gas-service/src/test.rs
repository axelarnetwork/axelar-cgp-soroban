#![cfg(test)]
extern crate std;

use crate::contract::{AxelarGasService, AxelarGasServiceClient};
use soroban_sdk::{bytes, bytesn, testutils::Address, Address, BytesN, Env, U256};

const DESTINATION_CHAIN: &str = "ethereum";
const DESTINATION_ADDRESS: &str = "0x4EFE356BEDeCC817cb89B4E9b796dB8bC188DC59";

fn setup_env<'a>() -> (Env, Address, AxelarGasServiceClient<'a>) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register_contract(None, AxelarGasService);
    
    //TODO: why am i getting this compilation error on "new"? "no function or associated item named `new` found for struct `AxelarGasService` in the current scope"
    let client = AxelarGasServiceClient::new(&env, &contract_id);

    client.initialize();

    (env, contract_id, client)
}

#[test]
fn pay_native_gas_for_contract_call() {
    let (env, contract_id, client) = setup_env();
    let sender: Address = Address::generate(&env);
    let refund_address: Address = Address::generate(&env);
    let payload = bytes!(&env, 0x1234);
    
    client.pay_native_gas_for_contract_call( sender, DESTINATION_CHAIN, DESTINATION_ADDRESS, &payload, refund_address);
    
}

#[test]
fn refund() {
    let (env, contract_id, client) = setup_env();
    
    let tx_hash: BytesN<32> = bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d);
    let log_index: U256 = U256::from_u32(&env, 1);
    let receiver: Address = Address::generate(&env);
    let token_addr: Address = Address::generate(&env);
    let amount: i128 = 1;
    
    client.refund(&tx_hash, &log_index, &receiver, &token_addr, &amount);
    
}