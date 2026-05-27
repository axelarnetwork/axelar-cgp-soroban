use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::xdr::ToXdr;
use stellar_axelar_std::{Address, BytesN, String};

use super::utils::setup_env;
use crate::storage;
use crate::types::TokenManagerType;
use crate::{deployer, InterchainTokenService};

const XRP_TOKEN_ID: [u8; 32] = [
    0xba, 0x5a, 0x21, 0xca, 0x88, 0xef, 0x6b, 0xba, 0x2b, 0xff, 0xf5, 0x08, 0x89, 0x94, 0xf9, 0x0e,
    0x10, 0x77, 0xe2, 0xa1, 0xcc, 0x3d, 0xcc, 0x38, 0xbd, 0x26, 0x1f, 0x00, 0xfc, 0xe2, 0x82, 0x4f,
];

/// The migration assumes an orphan TokenManager contract exists at the canonical
/// deterministic address (deployed by this ITS during a prior P2P registration).
/// To exercise that path in a unit test, we pre-deploy a TokenManager at the same
/// deterministic address using the canonical wasm — that lets the migration's
/// `UpgradableClient::upgrade` and follow-up `migrate` calls succeed against a
/// well-formed target. The migration then deploys the InterchainToken, sets the
/// TokenIdConfig, and runs `post_token_manager_deploy` (which adds the TokenManager
/// as a minter on the InterchainToken).
#[test]
fn migration_reconstructs_wxrp_state() {
    let (env, client, _gateway_client, _, _) = setup_env();

    let xrp_token_id = BytesN::<32>::from_array(&env, &XRP_TOKEN_ID);

    // Pre-condition: no TokenIdConfig.
    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, xrp_token_id.clone()).is_none());
    });

    // Pre-deploy a canonical TokenManager at the deterministic XRP address, simulating
    // the orphan left over from a P2P registration on a real network.
    let expected_token_manager = env.as_contract(&client.address, || -> Address {
        let token_manager_wasm = storage::token_manager_wasm_hash(&env);
        // Mirror the deployer's salt derivation so the deterministic address matches what
        // the migration's `deployer::token_manager_address` will resolve to.
        let salt: BytesN<32> = env
            .crypto()
            .keccak256(
                &(
                    String::from_str(&env, "its-token-manager-salt"),
                    xrp_token_id.clone(),
                )
                    .to_xdr(&env),
            )
            .into();
        env.deployer()
            .with_current_contract(salt)
            .deploy_v2(token_manager_wasm, (env.current_contract_address(),))
    });

    // Run the migration.
    client.mock_all_auths();
    env.as_contract(&client.address, || {
        InterchainTokenService::__migrate(&env, ()).unwrap();
    });

    // Post-condition: TokenIdConfig populated, pointing at the (now upgraded-in-place)
    // TokenManager and a freshly-deployed InterchainToken; type is NativeInterchainToken.
    env.as_contract(&client.address, || {
        let cfg = storage::try_token_id_config(&env, xrp_token_id.clone()).unwrap();
        assert_eq!(cfg.token_manager, expected_token_manager);
        assert_eq!(cfg.token_manager_type, TokenManagerType::NativeInterchainToken);
        // InterchainToken was newly deployed at the canonical deterministic address.
        assert_eq!(
            cfg.token_address,
            deployer::interchain_token_address(&env, xrp_token_id)
        );
    });
}
