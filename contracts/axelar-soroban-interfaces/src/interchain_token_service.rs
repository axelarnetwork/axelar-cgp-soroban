use soroban_sdk::{contractclient, contracterror, Address, Bytes, BytesN, Env, String};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum InterchainTokenServiceError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    NotOwner = 3,
    TrustedAddressAlreadyAdded = 4,
    NotTrustedAddress = 5,
}

/// Interface for the Interchain Token Service.
#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface {

    /// Initialize with owner address
    fn initialize(
        env: Env,
        owner: Address,
    ) -> Result<(), InterchainTokenServiceError>;

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
        decimals: u64, // changed to u64 as u8 was causing trait bound error - must fix
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
