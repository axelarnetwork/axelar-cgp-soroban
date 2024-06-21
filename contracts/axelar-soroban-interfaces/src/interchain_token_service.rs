use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

use crate::axelar_executable::AxelarExecutableInterface;

/// Interface for the Interchain Token Service.
#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    /// Initialize the gateway with the given auth module address.
    fn initialize(env: Env, auth_module: Address);

    /// Compute the interchain token id for the given deployer and salt.
    fn interchain_token_id(
        env: Env,
        deployer: Address,
        salt: BytesN<32>,
    ) -> BytesN<32>;

    /// Return the interchain token address registered to the given token id if it exists.
    fn valid_interchain_token_address(
        env: Env,
        token_id: BytesN<32>,
    ) -> BytesN<32>;

    /// Compute the interchain token address for the given token id if it was deployed on this chain.
    fn interchain_token_address(
        env: Env,
        token_id: BytesN<32>,
    ) -> BytesN<32>;

    /// Deploy a new interchain token to the destination chain with the given name, symbol, and decimals, and minter.
    /// If `destination_chain` is empty, the token is deployed to the current chain.
    /// If `minter` is empty, no minter is set for the token.
    fn deploy_interchain_token(
        env: Env,
        caller: Address,
        salt: BytesN<32>,
        destination_chain: String,
        name: String,
        symbol: String,
        decimals: u8,
        minter: Bytes,
    ) -> BytesN<32>;

    /// Transfer a token interchain to the given destination chain and address.
    fn interchain_transfer(
        env: Env,
        caller: Address,
        token_id: BytesN<32>,
        amount: i128,
        destination_chain: String,
        destination_address: String,
        metadata: Bytes,
    );
}
