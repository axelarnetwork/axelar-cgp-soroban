use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    fn owner(env: &Env) -> Address;

    fn transfer_ownership(env: &Env, new_owner: Address);

    fn trusted_address(env: &Env, chain: String) -> Option<String>;

    fn set_trusted_address(env: &Env, chain: String, address: String) -> Result<(), ContractError>;

    fn remove_trusted_address(env: &Env, chain: String) -> Result<(), ContractError>;

    #[allow(clippy::too_many_arguments)]
    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        destination_chain: String,
        name: String,
        symbol: String,
        decimals: u32,
        minter: Bytes,
        gas_token: Token,
    );

    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        destination_chain: String,
        token_id: String,
        gas_token: Token,
    );

    #[allow(clippy::too_many_arguments)]
    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: String,
        source_address: Bytes,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        metadata: Bytes,
        gas_token: Token,
    );
}
