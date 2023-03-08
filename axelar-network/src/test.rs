#![cfg(test)]

use super::*;
use soroban_sdk::{bytes, vec, Env};
extern crate std;

#[test]
fn test() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Contract);
    let client = ContractClient::new(&env, &contract_id);

    let params = ContractPayload {
        src_chain: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_add: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        contract: bytes!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        payload_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_tx_ha: bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d),
        src_evnt: 1 // source event index // do u256 instead?
    };

    let data: Data = Data {
        chain_id: 1,
        commandids: vec![&env, bytesn!(&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d)],
        commands: vec![&env, bytes![&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d]],
        params: vec![&env, params.serialize(&env)]
    };

    let input: Input = Input {
        data: data,
        proof: bytes![&env, 0xfded3f55dec47250a52a8c0bb7038e72fa6ffaae33562f77cd2b629ef7fd424d]
    };
}