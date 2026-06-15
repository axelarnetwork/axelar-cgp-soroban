use stellar_axelar_std::interfaces::CustomMigratableInterface;
use stellar_axelar_std::xdr::ToXdr;
use stellar_axelar_std::{Address, BytesN, Env, String};

use super::utils::setup_env;
use crate::types::TokenManagerType;
use crate::{deployer, storage, InterchainTokenService, RecoveryMigrationData};

/// Canonical XRP ITS token id (`0xba5a21ca…2824f`) — the mainnet wXRP repro case.
const XRP_TOKEN_ID: [u8; 32] = [
    0xba, 0x5a, 0x21, 0xca, 0x88, 0xef, 0x6b, 0xba, 0x2b, 0xff, 0xf5, 0x08, 0x89, 0x94, 0xf9, 0x0e,
    0x10, 0x77, 0xe2, 0xa1, 0xcc, 0x3d, 0xcc, 0x38, 0xbd, 0x26, 0x1f, 0x00, 0xfc, 0xe2, 0x82, 0x4f,
];

/// An arbitrary other token id, used to prove the migration body is genuinely
/// parameterized and not implicitly coupled to the wXRP constants. Any non-wXRP
/// value works — choosing one that's distinct in every byte makes failures obvious.
const REPRO_TOKEN_ID: [u8; 32] = [
    0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef,
    0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef,
];

/// Pre-deploy a canonical TokenManager at the deterministic address for `token_id`,
/// simulating the orphan left over from a P2P registration on a real network.
/// Returns the deterministic TokenManager address the migration will resolve to.
fn pre_deploy_orphan_token_manager(
    env: &Env,
    its_address: &Address,
    token_id: &BytesN<32>,
) -> Address {
    env.as_contract(its_address, || -> Address {
        let token_manager_wasm = storage::token_manager_wasm_hash(env);
        // Mirror the deployer's salt derivation so the deterministic address matches
        // what the migration's `deployer::token_manager_address` will resolve to.
        let salt: BytesN<32> = env
            .crypto()
            .keccak256(
                &(
                    String::from_str(env, "its-token-manager-salt"),
                    token_id.clone(),
                )
                    .to_xdr(env),
            )
            .into();
        env.deployer()
            .with_current_contract(salt)
            .deploy_v2(token_manager_wasm, (env.current_contract_address(),))
    })
}

/// Mainnet repro: wXRP token_id + canonical Wrapped XRP / wXRP / 6 metadata.
///
/// The migration assumes an orphan TokenManager contract exists at the canonical
/// deterministic address (deployed by this ITS during a prior P2P registration).
/// We pre-deploy a TokenManager at the same deterministic address using the
/// canonical wasm — that lets the migration's `UpgradableClient::upgrade` and
/// follow-up `migrate` calls succeed against a well-formed target. The migration
/// then deploys the InterchainToken, sets the TokenIdConfig, and runs
/// `post_token_manager_deploy` (which adds the TokenManager as a minter on the IT).
#[test]
fn migration_reconstructs_wxrp_state() {
    let (env, client, _gateway_client, _, _) = setup_env();

    let xrp_token_id = BytesN::<32>::from_array(&env, &XRP_TOKEN_ID);

    // Pre-condition: no TokenIdConfig.
    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, xrp_token_id.clone()).is_none());
    });

    let expected_token_manager =
        pre_deploy_orphan_token_manager(&env, &client.address, &xrp_token_id);

    // Run the migration with the mainnet wXRP params.
    client.mock_all_auths();
    env.as_contract(&client.address, || {
        InterchainTokenService::__migrate(
            &env,
            RecoveryMigrationData {
                token_decimals: 6,
                token_id: xrp_token_id.clone(),
                token_name: String::from_str(&env, "Wrapped XRP"),
                token_symbol: String::from_str(&env, "wXRP"),
            },
        )
        .unwrap();
    });

    // Post-condition: TokenIdConfig populated, pointing at the (now upgraded-in-place)
    // TokenManager and a freshly-deployed InterchainToken; type is NativeInterchainToken.
    env.as_contract(&client.address, || {
        let cfg = storage::try_token_id_config(&env, xrp_token_id.clone()).unwrap();
        assert_eq!(cfg.token_manager, expected_token_manager);
        assert_eq!(
            cfg.token_manager_type,
            TokenManagerType::NativeInterchainToken
        );
        // InterchainToken was newly deployed at the canonical deterministic address.
        assert_eq!(
            cfg.token_address,
            deployer::interchain_token_address(&env, xrp_token_id)
        );
    });
}

/// Devnet/testnet repro: same migration body, different token_id + metadata.
/// Exists to prove the migration is genuinely parameterized — i.e. the wXRP case
/// isn't implicitly coupled to constants somewhere — and to give devnet/testnet
/// validation a non-wXRP path it can reproduce without needing the real XRPL
/// gateway's instantiation parameters.
#[test]
fn migration_reconstructs_arbitrary_token_state() {
    let (env, client, _gateway_client, _, _) = setup_env();

    let repro_token_id = BytesN::<32>::from_array(&env, &REPRO_TOKEN_ID);

    env.as_contract(&client.address, || {
        assert!(storage::try_token_id_config(&env, repro_token_id.clone()).is_none());
    });

    let expected_token_manager =
        pre_deploy_orphan_token_manager(&env, &client.address, &repro_token_id);

    client.mock_all_auths();
    env.as_contract(&client.address, || {
        InterchainTokenService::__migrate(
            &env,
            RecoveryMigrationData {
                token_decimals: 18,
                token_id: repro_token_id.clone(),
                token_name: String::from_str(&env, "Devnet Repro Token"),
                token_symbol: String::from_str(&env, "REPRO"),
            },
        )
        .unwrap();
    });

    env.as_contract(&client.address, || {
        let cfg = storage::try_token_id_config(&env, repro_token_id.clone()).unwrap();
        assert_eq!(cfg.token_manager, expected_token_manager);
        assert_eq!(
            cfg.token_manager_type,
            TokenManagerType::NativeInterchainToken
        );
        assert_eq!(
            cfg.token_address,
            deployer::interchain_token_address(&env, repro_token_id)
        );
    });
}
