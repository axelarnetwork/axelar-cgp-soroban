#![cfg(test)]
extern crate std;

use axelar_soroban_std::{
    assert_invoke_auth_err, assert_invoke_auth_ok, assert_last_emitted_event,
};
use interchain_token::{contract::InterchainToken, InterchainTokenClient};
use soroban_sdk::testutils::{MockAuth, MockAuthInvoke};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _},
    Address, BytesN, Env, IntoVal as _, Symbol,
};
use soroban_token_sdk::metadata::TokenMetadata;

fn create_token<'a>(env: &Env, owner: &Address, minter: &Address) -> InterchainTokenClient<'a> {
    let interchain_token_service = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_meta_data = TokenMetadata {
        decimal: 6,
        name: "name".into_val(env),
        symbol: "symbol".into_val(env),
    };

    let contract_id = env.register(
        InterchainToken,
        (
            owner,
            minter,
            &interchain_token_service,
            &token_id,
            token_meta_data,
        ),
    );

    InterchainTokenClient::new(env, &contract_id)
}

#[test]
fn test() {
    let env = Env::default();

    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let token = create_token(&env, &owner1, &minter1);

    assert_invoke_auth_ok!(minter1, token.try_mint(&minter1, &user1, &1000_i128));
    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (minter1, &user1, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 1000_i128);

    let expiration_ledger = 200;

    assert_invoke_auth_ok!(
        user2,
        token.try_approve(&user2, &user3, &500_i128, &expiration_ledger)
    );
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 500_i128, expiration_ledger).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 500_i128);

    assert_invoke_auth_ok!(user1, token.try_transfer(&user1, &user2, &600_i128));
    assert_eq!(
        env.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("transfer"),
                    (&user1, &user2, 600_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 400_i128);
    assert_eq!(token.balance(&user2), 600_i128);

    assert_invoke_auth_ok!(
        user3,
        token.try_transfer_from(&user3, &user2, &user1, &400_i128)
    );
    assert_eq!(
        env.auths(),
        std::vec![(
            user3.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&env, "transfer_from"),
                    (&user3, &user2, &user1, 400_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 800_i128);
    assert_eq!(token.balance(&user2), 200_i128);

    assert_invoke_auth_ok!(user1, token.try_transfer(&user1, &user3, &300_i128));
    assert_eq!(token.balance(&user1), 500_i128);
    assert_eq!(token.balance(&user3), 300_i128);

    assert_invoke_auth_ok!(owner1, token.try_transfer_ownership(&owner2));
    assert_eq!(
        env.auths(),
        std::vec![(
            owner1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    Symbol::new(&env, "transfer_ownership"),
                    (&owner2,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Increase to 500
    assert_invoke_auth_ok!(
        user2,
        token.try_approve(&user2, &user3, &500_i128, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user2, &user3), 500_i128);

    assert_invoke_auth_ok!(
        user2,
        token.try_approve(&user2, &user3, &0_i128, &expiration_ledger)
    );
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 0_i128, expiration_ledger).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 0);
}

#[test]
fn minter_test() {
    let env = Env::default();

    let amount = 1000;
    let owner = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let token = create_token(&env, &owner, &minter1);

    assert_invoke_auth_err!(owner, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_err!(user, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_ok!(minter1, token.try_mint(&minter1, &user, &amount));

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (minter1, &user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount);
}

#[test]
fn add_minter() {
    let env = Env::default();

    let amount = 1000;
    let owner = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let minter2 = Address::generate(&env);
    let user = Address::generate(&env);

    let token = create_token(&env, &owner, &minter1);

    assert_invoke_auth_err!(owner, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_err!(user, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_ok!(minter1, token.try_mint(&minter1, &user, &amount));

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (minter1, &user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount);

    assert_invoke_auth_ok!(owner, token.try_add_minter(&minter2));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_added"), minter2.clone()),
        (),
    );

    assert_invoke_auth_ok!(minter2, token.try_mint(&minter2, &user, &amount));

    assert_eq!(
        env.auths(),
        std::vec![(
            minter2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (minter2, &user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount * 2);
}

#[test]
fn remove_minter() {
    let env = Env::default();

    let amount = 1000;
    let owner = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let token = create_token(&env, &owner, &minter1);

    assert_invoke_auth_err!(owner, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_err!(user, token.try_mint(&minter1, &user, &amount));
    assert_invoke_auth_ok!(minter1, token.try_mint(&minter1, &user, &amount));

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (minter1.clone(), &user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount);

    assert_invoke_auth_ok!(owner, token.try_remove_minter(&minter1));

    assert_last_emitted_event(
        &env,
        &token.address,
        (Symbol::new(&env, "minter_removed"), minter1.clone()),
        (),
    );

    assert_invoke_auth_err!(minter1, token.try_mint(&minter1, &user, &amount));
}

#[test]
fn test_burn() {
    let env = Env::default();

    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &owner, &minter);
    let amount = 1000;

    assert_invoke_auth_ok!(minter, token.try_mint(&minter, &user1, &amount));
    assert_eq!(token.balance(&user1), amount);

    let expiration_ledger = 200;
    let amount2 = 500;

    assert_invoke_auth_ok!(
        user1,
        token.try_approve(&user1, &user2, &amount2, &expiration_ledger)
    );
    assert_eq!(token.allowance(&user1, &user2), amount2);

    assert_invoke_auth_ok!(user2, token.try_burn_from(&user2, &user1, &amount2));
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn_from"),
                    (&user2, &user1, amount2).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), amount2);
    assert_eq!(token.balance(&user2), 0);

    assert_invoke_auth_ok!(user1, token.try_burn(&user1, &amount2));
    assert_eq!(
        env.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, amount2).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn decimal_is_over_max() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_meta_data = TokenMetadata {
        decimal: (u32::from(u8::MAX) + 1),
        name: "name".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            owner,
            minter,
            &interchain_token_service,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_name_is_empty() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            owner,
            minter,
            &interchain_token_service,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_symbol_is_empty() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            owner,
            minter,
            &interchain_token_service,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_id_is_empty() {
    let env = Env::default();
    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: BytesN<32> = BytesN::<32>::random(&env);
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            owner,
            minter,
            &interchain_token_service,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &owner, &minter);

    token.mint(&minter, &user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn transfer_from_insufficient_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let owner = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let token = create_token(&env, &owner, &minter);

    token.mint(&minter, &user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user3, &100, &200);
    assert_eq!(token.allowance(&user1, &user3), 100);

    token.transfer_from(&user3, &user1, &user2, &101);
}
