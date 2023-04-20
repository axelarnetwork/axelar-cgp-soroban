use soroban_sdk::{contractimpl, contracttype, contracterror, bytes, Bytes, BytesN, Env, Symbol, vec, Address, Map, map, Vec, crypto, bytesn,
    xdr::{self, FromXdr, ToXdr}, panic_with_error, String
};

mod gateway {
    soroban_sdk::contractimport!(
        file = "../../target/wasm32-unknown-unknown/release/contract.wasm"
    );
}

pub struct Executable;

#[contractimpl]
impl Executable {

    pub fn execute(
        env: Env,
        contract_id: BytesN<32>,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload: Bytes
    ) {
        let client = gateway::Client::new(&env, &contract_id);
        let payload_hash: BytesN<32> = env.crypto().sha256(&payload);


        

    }

}