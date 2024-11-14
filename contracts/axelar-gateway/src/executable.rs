use core::panic;

use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::AxelarGatewayClient;

/// Interface for an Axelar Executable app.
#[contractclient(name = "AxelarExecutableClient")]
pub trait AxelarExecutableInterface {
    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain contract call with the given payload. This function must validate that the contract call is received from the trusted gateway.
    fn execute(
        env: Env,
        source_chain: String,
        message_id: String,
        source_address: String,
        payload: Bytes,
    );

    /// Validate if a gateway has approved a contract call.
    /// This should be called from an implementation of `execute` before executing custom app logic.
    /// This method doesn't get exposed from the contract, as Soroban SDK's contractimpl macro ignores default trait methods.
    fn validate(
        env: &Env,
        source_chain: &String,
        message_id: &String,
        source_address: &String,
        payload: &Bytes,
    ) {
        let gateway = AxelarGatewayClient::new(env, &Self::gateway(env));

        // Validate the contract call was approved by the gateway
        if !gateway.validate_message(
            &env.current_contract_address(),
            source_chain,
            message_id,
            source_address,
            &env.crypto().keccak256(payload).into(),
        ) {
            panic!("not approved");
        };
    }
}
