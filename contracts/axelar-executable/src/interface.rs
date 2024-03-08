use soroban_sdk::{contractclient, panic_with_error, Address, Bytes, BytesN, Env, String};

use axelar_gateway::contract::AxelarGatewayClient;

pub trait AxelarExecutableInternal {
    fn execute_internal(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    );
}

/// Interface for an Axelar Executable app.
#[contractclient(name = "AxelarExecutableClient")]
pub trait AxelarExecutableInterface {
    type Internal: AxelarExecutableInternal;

    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain contract call with the given payload. This function must validate that the contract call is received from the trusted gateway.
    fn execute(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    ) {
        let gateway = AxelarGatewayClient::new(&env, &Self::gateway(&env));

        // Validate the contract call was approved by the gateway
        if !gateway.validate_contract_call(
            &env.current_contract_address(),
            &command_id,
            &source_chain,
            &source_address,
            &env.crypto().keccak256(&payload),
        ) {
            panic_with_error!(env, crate::error::Error::NotApproved);
        };

        Self::Internal::execute_internal(env, command_id, source_chain, source_address, payload);
    }
}
