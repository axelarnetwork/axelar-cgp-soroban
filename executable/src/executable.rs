use soroban_sdk::{contractimpl, contracttype, contractclient, contracterror, bytes, Bytes, BytesN, Env, Symbol, vec, Address, Map, map, Vec, crypto, bytesn,
    xdr::{self, FromXdr, ToXdr}, panic_with_error, String
};

mod gateway {
    soroban_sdk::contractimport!(
        file = "../contract/target/wasm32-unknown-unknown/release/contract.wasm"
    );
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotApprovedByGateway = 8,
}

/// This is a trait that is implemented by the contract and provides a contract-specific way to execute a command.
pub trait ContractExecutable {
    fn _execute(env: Env, source_chain: String, source_address: String, payload: Bytes);
}

pub trait Executable {
    fn execute(
        env: Env,
        contract_id: BytesN<32>,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        contract_address: String, // because soroban does not have msg.sender
        payload: Bytes
    );

}

/// A macro that is used to implement the AxelarExecutable trait for the contract.
#[macro_export]
macro_rules! impl_axelar_executable {
    ($contract: ident, $contract_id: ident, $_execute: ident) => {

        #[contractclient(name = "ExecuteClient")]
        pub trait ExecuteInteface {
        fn _execute(env: Env, source_chain: String, source_address: String, payload: Bytes);
        }

        #[contractimpl]
        impl Executable for $contract {
        
            pub fn execute(
                env: Env,
                gateway_contract_id: BytesN<32>,
                command_id: BytesN<32>,
                source_chain: String,
                source_address: String,
                contract_address: String, // because soroban does not have msg.sender
                payload: Bytes
            ) {
                let client = gateway::Client::new(&env, &gateway_contract_id);
                let payload_hash: BytesN<32> = env.crypto().sha256(&payload);
        
                if (!client.validate_contract_call(&command_id, &source_chain, &source_address, &contract_address, &payload_hash)) {
                    panic_with_error!(env, Error::NotApprovedByGateway);
                }

                let execute_client = ExecuteClient::new(&env, $contract_id)

                execute_client._execute(source_chain, source_address, payload);

            }
        
        }

    }
}