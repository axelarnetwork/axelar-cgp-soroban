use soroban_sdk::{contractclient, token, Address, BytesN, Env};

use crate::error::ContractError;

#[allow(dead_code)]
#[contractclient(name = "InterchainTokenClient")]
pub trait InterchainTokenInterface: token::Interface {
    fn token_id(env: &Env) -> BytesN<32>;
    fn interchain_token_service(env: &Env) -> Address;

    fn is_minter(env: &Env, minter: Address) -> bool;
    fn mint(env: Env, minter: Address, to: Address, amount: i128) -> Result<(), ContractError>;
    fn add_minter(env: &Env, minter: Address);
    fn remove_minter(env: &Env, minter: Address);
}
