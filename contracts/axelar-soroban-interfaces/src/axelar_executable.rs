use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

/// Interface for an Axelar Executable app.
#[contractclient(name = "AxelarExecutableClient")]
pub trait AxelarExecutableInterface {
    /// Return the trusted gateway contract id.
    fn gateway(env: &Env) -> Address;

    /// Execute a cross-chain contract call with the given payload. This function must validate that the contract call is received from the trusted gateway.
    fn execute(
        env: Env,
        command_id: BytesN<32>,
        source_chain: String,
        source_address: String,
        payload: Bytes,
    );
}
