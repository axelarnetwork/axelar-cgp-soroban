#![cfg(test)]
extern crate std;

use axelar_soroban_std::assert_contract_err;
use interchain_token::{contract::InterchainToken, error::ContractError, InterchainTokenClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _},
    Address, Bytes, BytesN, Env, IntoVal as _, Symbol,
};

fn create_token<'a>(env: &Env, admin: &Address, minter: &Address) -> InterchainTokenClient<'a> {
    let token = InterchainTokenClient::new(env, &env.register_contract(None, InterchainToken {}));
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let decimal = 6;

    token.initialize_interchain_token(
        &interchain_token_service,
        admin,
        &minter,
        &token_id,
        &decimal,
        &"name".into_val(env),
        &"symbol".into_val(env),
    );
    token
}

#[test]
fn test() {
    let env = Env::default();
    env.mock_all_auths();

    let admin1 = Address::generate(&env);
    let admin2 = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);

    let token = create_token(&env, &admin1, &minter1);

    token.mint(&admin1, &user1, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user1, 1000_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user2, &user3, &500, &200);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 500_i128, 200_u32).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.allowance(&user2, &user3), 500);

    token.transfer(&user1, &user2, &600);
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
    assert_eq!(token.balance(&user1), 400);
    assert_eq!(token.balance(&user2), 600);

    token.transfer_from(&user3, &user2, &user1, &400);
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
    assert_eq!(token.balance(&user1), 800);
    assert_eq!(token.balance(&user2), 200);

    token.transfer(&user1, &user3, &300);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user3), 300);

    token.set_admin(&admin2);
    assert_eq!(
        env.auths(),
        std::vec![(
            admin1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("set_admin"),
                    (&admin2,).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    // Increase to 500
    token.approve(&user2, &user3, &500, &200);
    assert_eq!(token.allowance(&user2, &user3), 500);
    token.approve(&user2, &user3, &0, &200);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("approve"),
                    (&user2, &user3, 0_i128, 200_u32).into_val(&env),
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
    env.mock_all_auths();

    let amount = 1000;
    let admin = Address::generate(&env);
    let minter1 = Address::generate(&env);
    let user = Address::generate(&env);

    let token = create_token(&env, &admin, &minter1);

    // Admin can mint token to user
    token.mint(&admin, &user, &amount);

    assert_eq!(
        env.auths(),
        std::vec![(
            admin.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount);

    // minter can mint token to user
    token.mint(&minter1, &user, &amount);

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("mint"),
                    (&user, amount).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
    assert_eq!(token.balance(&user), amount * 2);
}

#[test]
fn test_burn() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin, &minter);

    token.mint(&minter, &user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user2, &500, &200);
    assert_eq!(token.allowance(&user1, &user2), 500);

    token.burn_from(&user2, &user1, &500);
    assert_eq!(
        env.auths(),
        std::vec![(
            user2.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn_from"),
                    (&user2, &user1, 500_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.allowance(&user1, &user2), 0);
    assert_eq!(token.balance(&user1), 500);
    assert_eq!(token.balance(&user2), 0);

    token.burn(&user1, &500);
    assert_eq!(
        env.auths(),
        std::vec![(
            user1.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    token.address.clone(),
                    symbol_short!("burn"),
                    (&user1, 500_i128).into_val(&env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );

    assert_eq!(token.balance(&user1), 0);
    assert_eq!(token.balance(&user2), 0);
}

#[test]
fn initialize_already_initialized() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);

    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let decimal = 6;

    let token = create_token(&env, &admin, &minter);

    assert_contract_err!(
        token.try_initialize_interchain_token(
            &interchain_token_service,
            &admin,
            &minter,
            &token_id,
            &decimal,
            &"name".into_val(&env),
            &"symbol".into_val(&env),
        ),
        ContractError::AlreadyInitialized
    )
}

#[test]
fn decimal_is_over_eighteen() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let token = InterchainTokenClient::new(&env, &env.register_contract(None, InterchainToken {}));

    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let decimal = 19;

    assert_contract_err!(
        token.try_initialize_interchain_token(
            &interchain_token_service,
            &admin,
            &minter,
            &token_id,
            &decimal,
            &"name".into_val(&env),
            &"symbol".into_val(&env),
        ),
        ContractError::InvalidDecimal
    )
}

#[test]
#[should_panic(expected = "insufficient balance")]
fn transfer_insufficient_balance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let token = create_token(&env, &admin, &minter);

    token.mint(&minter, &user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.transfer(&user1, &user2, &1001);
}

#[test]
#[should_panic(expected = "insufficient allowance")]
fn transfer_from_insufficient_allowance() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);
    let user3 = Address::generate(&env);
    let token = create_token(&env, &admin, &minter);

    token.mint(&minter, &user1, &1000);
    assert_eq!(token.balance(&user1), 1000);

    token.approve(&user1, &user3, &100, &200);
    assert_eq!(token.allowance(&user1, &user3), 100);

    token.transfer_from(&user3, &user1, &user2, &101);
}
