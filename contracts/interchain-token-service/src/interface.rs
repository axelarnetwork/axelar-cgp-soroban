use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    fn owner(env: &Env) -> Address;

    fn transfer_ownership(env: &Env, new_owner: Address);

    fn trusted_address(env: &Env, chain: String) -> Option<String>;

    fn set_trusted_address(env: &Env, chain: String, address: String) -> Result<(), ContractError>;

    fn remove_trusted_address(env: &Env, chain: String) -> Result<(), ContractError>;

    fn deploy_interchain_token(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        name: String,
        symbol: String,
        decimals: u32,
        minter: Option<Bytes>,
        gas_token: Token,
    );

    fn deploy_remote_interchain_token(
        _env: &Env,
        _caller: Address,
        _destination_chain: String,
        _token_id: String,
        _gas_token: Token,
    );

    fn interchain_transfer(
        _env: &Env,
        _caller: Address,
        _token_id: BytesN<32>,
        _destination_chain: String,
        _destination_address: Bytes,
        _amount: i128,
        _metadata: Bytes,
        _gas_token: Token,
    );
}
