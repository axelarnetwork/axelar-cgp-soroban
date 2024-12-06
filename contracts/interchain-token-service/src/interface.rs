use axelar_gateway::executable::AxelarExecutableInterface;
use axelar_soroban_std::types::Token;
use soroban_sdk::{contractclient, Address, Bytes, BytesN, Env, String};
use soroban_token_sdk::metadata::TokenMetadata;

use crate::error::ContractError;

#[contractclient(name = "InterchainTokenServiceClient")]
pub trait InterchainTokenServiceInterface: AxelarExecutableInterface {
    fn transfer_ownership(env: &Env, new_owner: Address);

    fn trusted_address(env: &Env, chain: String) -> Option<String>;

    fn set_trusted_address(env: &Env, chain: String, address: String) -> Result<(), ContractError>;

    fn remove_trusted_address(env: &Env, chain: String) -> Result<(), ContractError>;

    fn interchain_token_deploy_salt(env: &Env, deployer: Address, salt: BytesN<32>) -> BytesN<32>;

    fn interchain_token_id(env: &Env, sender: Address, salt: BytesN<32>) -> BytesN<32>;

    fn deploy_interchain_token(
        env: &Env,
        deployer: Address,
        salt: BytesN<32>,
        token_meta_data: TokenMetadata,
        initial_supply: i128,
        _minter: Address,
    ) -> Result<BytesN<32>, ContractError>;

    fn deploy_remote_interchain_token(
        env: &Env,
        caller: Address,
        destination_chain: String,
        _token_id: String,
        gas_token: Token,
    );

    fn interchain_transfer(
        env: &Env,
        caller: Address,
        _token_id: String,
        _source_address: Bytes,
        destination_chain: String,
        destination_address: Bytes,
        _amount: i128,
        _metadata: Bytes,
        gas_token: Token,
    );
}
