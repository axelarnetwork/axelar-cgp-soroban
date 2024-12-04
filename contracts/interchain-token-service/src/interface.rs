use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    fn gas_service(env: &Env) -> Address;

    fn its_hub_routing_identifier(env: &Env) -> String;

    fn its_hub_chain_name(env: &Env) -> String;

    fn owner(env: &Env) -> Address;

    fn transfer_ownership(env: &Env, new_owner: Address);

    fn trusted_address(env: &Env, chain: String) -> Option<String>;

    fn set_trusted_address(env: &Env, chain: String, address: String) -> Result<(), ContractError>;

    fn remove_trusted_address(env: &Env, chain: String) -> Result<(), ContractError>;

    fn deploy_interchain_token(
        _env: &Env,
        _caller: Address,
        _token_id: BytesN<32>,
        _destination_chain: String,
        _name: String,
        _symbol: String,
        _decimals: u32,
        _minter: Option<Bytes>,
        _gas_token: Token,
    );

    fn deploy_remote_interchain_token(
        _env: &Env,
        _caller: Address,
        _destination_chain: String,
        _token_id: String,
        _gas_token: Token,
    );

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        token_id: BytesN<32>,
        destination_chain: String,
        destination_address: Bytes,
        amount: i128,
        metadata: Option<Bytes>,
        gas_token: Token,
    );
}
