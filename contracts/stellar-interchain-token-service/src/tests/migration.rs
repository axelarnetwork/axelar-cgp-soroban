use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::testutils::Address as _;
use stellar_axelar_std::{Address, BytesN};

use super::utils::setup_env;
use crate::types::TokenManagerType;
use crate::{storage, InterchainTokenService};

const XRP_TOKEN_ID: [u8; 32] = [
    0xba, 0x5a, 0x21, 0xca, 0x88, 0xef, 0x6b, 0xba, 0x2b, 0xff, 0xf5, 0x08, 0x89, 0x94, 0xf9, 0x0e,
    0x10, 0x77, 0xe2, 0xa1, 0xcc, 0x3d, 0xcc, 0x38, 0xbd, 0x26, 0x1f, 0x00, 0xfc, 0xe2, 0x82, 0x4f,
];

#[test]
fn migration_removes_xrp_stellar_token_config() {
    let (env, client, _, _, _) = setup_env();

    let deployer = Address::generate(&env);
    let token = env.register_stellar_asset_contract_v2(deployer.clone());
    let salt = BytesN::<32>::from_array(&env, &[99; 32]);

    // Register a custom token so a TokenIdConfig is created
    client.mock_all_auths().register_custom_token(
        &deployer,
        &salt,
        &token.address(),
        &TokenManagerType::LockUnlock,
    );
    let other_token_id = client.linked_token_id(&deployer, &salt);

    let xrp_token_id = BytesN::<32>::from_array(&env, &XRP_TOKEN_ID);

    // Set up XRP token config and flow limit within the contract context
    env.as_contract(&client.address, || {
        storage::set_token_id_config(
            &env,
            xrp_token_id.clone(),
            &storage::TokenIdConfigValue {
                token_address: Address::generate(&env),
                token_manager: Address::generate(&env),
                token_manager_type: TokenManagerType::MintBurnFrom,
            },
        );
        storage::set_flow_limit(&env, xrp_token_id.clone(), &0);
    });

    // Verify both exist before migration
    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, xrp_token_id.clone()).is_some());
        assert!(storage::try_flow_limit(&env, xrp_token_id.clone()).is_some());
        assert!(storage::try_token_id_config(&env, other_token_id.clone()).is_some());
    });

    // Run migration
    env.as_contract(&client.address, || {
        InterchainTokenService::__migrate(&env, ()).unwrap();
    });

    // XRP token config and flow limit should be removed, other token preserved
    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, xrp_token_id.clone()).is_none());
        assert!(storage::try_flow_limit(&env, xrp_token_id).is_none());
        assert!(storage::try_token_id_config(&env, other_token_id).is_some());
    });
}

#[test]
fn migration_succeeds_when_xrp_token_not_registered() {
    let (env, client, _, _, _) = setup_env();

    let xrp_token_id = BytesN::<32>::from_array(&env, &XRP_TOKEN_ID);

    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, xrp_token_id.clone()).is_none());

        InterchainTokenService::__migrate(&env, ()).unwrap();

        assert!(storage::try_token_id_config(&env, xrp_token_id).is_none());
    });
}
