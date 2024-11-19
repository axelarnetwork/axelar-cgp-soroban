#![cfg(test)]
extern crate std;

//use axelar_soroban_std::assert_contract_err;
use interchain_token::{contract::InterchainToken, InterchainTokenClient};
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation, BytesN as _},
    Address, Bytes, BytesN, Env, IntoVal as _, Symbol,
};
use soroban_token_sdk::metadata::TokenMetadata;

fn create_token<'a>(env: &Env, admin: &Address, minter: &Address) -> InterchainTokenClient<'a> {
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 6,
        name: "name".into_val(env),
        symbol: "symbol".into_val(env),
    };

    let contract_id = env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
    let token = InterchainTokenClient::new(env, &contract_id);

    let token = InterchainTokenClient::new(env, &contract_id);

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

    token.mint(&minter1, &user1, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
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
    assert_eq!(token.balance(&user), amount);
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
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn decimal_is_over_max() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: (u32::from(u8::MAX) + 1),
        name: "name".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_name_is_empty() {
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_name_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_symbol_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_symbol_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_id_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::from_array(&env, &[]).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

fn create_token<'a>(env: &Env, admin: &Address, minter: &Address) -> InterchainTokenClient<'a> {
    let token = InterchainTokenClient::new(env, &env.register_contract(None, InterchainToken {}));
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 6,
        name: "name".into_val(env),
        symbol: "symbol".into_val(env),
    };

    let contract_id = env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
    let token = InterchainTokenClient::new(env, &contract_id);

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

    token.mint(&minter1, &user1, &1000);

    assert_eq!(
        env.auths(),
        std::vec![(
            minter1.clone(),
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
    assert_eq!(token.balance(&user), amount);
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
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn decimal_is_over_max() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: (u32::from(u8::MAX) + 1),
        name: "name".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_name_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_symbol_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::<20>::random(&env).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
    );
}

#[test]
#[should_panic(expected = "HostError: Error(Context, InvalidAction)")]
fn token_id_is_empty() {
    let env = Env::default();
    let admin = Address::generate(&env);
    let minter = Address::generate(&env);
    let interchain_token_service = Address::generate(&env);
    let token_id: Bytes = BytesN::from_array(&env, &[]).into();
    let token_meta_data = TokenMetadata {
        decimal: 1,
        name: "name".into_val(&env),
        symbol: "symbol".into_val(&env),
    };

    env.register(
        InterchainToken,
        (
            &interchain_token_service,
            admin,
            minter,
            &token_id,
            token_meta_data,
        ),
        ContractError::InvalidDecimal
    );

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
