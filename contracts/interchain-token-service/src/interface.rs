use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, Env, String};

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface {
    fn owner(env: &Env) -> Address;

    fn transfer_ownership(env: Env, new_owner: Address);

    fn trusted_address(env: &Env, chain: String) -> Option<String>;

    fn set_trusted_address(env: Env, chain: String, address: String) -> Result<(), ContractError>;

    fn remove_trusted_address(env: Env, chain: String) -> Result<(), ContractError>;

    fn execute_message(
        _env: &Env,
        _source_chain: String,
        _message_id: String,
        _source_address: String,
        _payload: Bytes,
    ) -> Result<(), ContractError>;

    #[allow(clippy::too_many_arguments)]
    fn deploy_interchain_token(
        _env: Env,
        _caller: Address,
        _destination_chain: String,
        _name: String,
        _symbol: String,
        _decimals: i128,
        _minter: Bytes,
        _gas_token: Token,
    );

    fn deploy_remote_interchain_token(
        env: Env,
        caller: Address,
        destination_chain: String,
        _token_id: String,
        gas_token: Token,
        _metadata: Bytes,
    );

    #[allow(clippy::too_many_arguments)]
    fn interchain_token_transfer(
        env: Env,
        caller: Address,
        _token_id: String,
        destination_chain: String,
        destination_address: String,
        _amount: i128,
        _metadata: Bytes,
        gas_token: Token,
    );
}
