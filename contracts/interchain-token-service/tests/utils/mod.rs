use axelar_gas_service::{AxelarGasService, AxelarGasServiceClient};
use axelar_gateway::testutils::{setup_gateway, TestSignerSet};
use axelar_gateway::AxelarGatewayClient;
use axelar_soroban_std::types::Token;
use interchain_token_service::{InterchainTokenService, InterchainTokenServiceClient};
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

pub const HUB_CHAIN: &str = "hub_chain";
const HUB_ADDRESS: &str = "hub_address";

const INTERCHAIN_TOKEN_WASM_HASH: &[u8] = include_bytes!("../testdata/interchain_token.wasm");

pub fn setup_gas_service<'a>(env: &Env) -> AxelarGasServiceClient<'a> {
    let owner: Address = Address::generate(env);
    let gas_collector: Address = Address::generate(env);
    let gas_service_id = env.register(AxelarGasService, (&owner, &gas_collector));
    let gas_service_client = AxelarGasServiceClient::new(env, &gas_service_id);

    gas_service_client
}

fn setup_its<'a>(
    env: &Env,
    gateway: &AxelarGatewayClient,
    gas_service: &AxelarGasServiceClient,
) -> InterchainTokenServiceClient<'a> {
    let owner = Address::generate(env);
    let chain_name = String::from_str(env, "chain_name");
    let interchain_token_wasm_hash = env
        .deployer()
        .upload_contract_wasm(INTERCHAIN_TOKEN_WASM_HASH);

    let contract_id = env.register(
        InterchainTokenService,
        (
            &owner,
            &gateway.address,
            &gas_service.address,
            chain_name,
            interchain_token_wasm_hash,
        ),
    );

    InterchainTokenServiceClient::new(env, &contract_id)
}

pub fn setup_env<'a>() -> (
    Env,
    InterchainTokenServiceClient<'a>,
    AxelarGatewayClient<'a>,
    TestSignerSet,
) {
    let env = Env::default();

    let (signers, gateway_client) = setup_gateway(&env, 0, 5);
    let gas_service_client = setup_gas_service(&env);

    let client = setup_its(&env, &gateway_client, &gas_service_client);

    (env, client, gateway_client, signers)
}

#[allow(dead_code)]
pub fn setup_gas_token(env: &Env, sender: &Address) -> Token {
    let asset = &env.register_stellar_asset_contract_v2(Address::generate(env));
    let gas_amount: i128 = 1;
    let gas_token = Token {
        address: asset.address(),
        amount: gas_amount,
    };

    StellarAssetClient::new(env, &asset.address())
        .mock_all_auths()
        .mint(sender, &gas_amount);

    gas_token
}

#[allow(dead_code)]
pub fn register_chains(env: &Env, client: &InterchainTokenServiceClient) {
    let chain = String::from_str(env, HUB_CHAIN);
    client
        .mock_all_auths()
        .set_trusted_address(&chain, &client.its_hub_routing_identifier());

    let chain = client.its_hub_chain_name();
    let addr = String::from_str(env, HUB_ADDRESS);
    client.mock_all_auths().set_trusted_address(&chain, &addr);
}
