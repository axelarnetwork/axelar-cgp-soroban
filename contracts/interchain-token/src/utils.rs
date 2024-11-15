use soroban_sdk::{log, Address, Env, String};

use crate::storage_types::{
    AllowanceDataKey, AllowanceValue, DataKey, BALANCE_BUMP_AMOUNT, BALANCE_LIFETIME_THRESHOLD,
    INSTANCE_BUMP_AMOUNT, INSTANCE_LIFETIME_THRESHOLD,
};
use soroban_token_sdk::metadata::TokenMetadata;
use soroban_token_sdk::TokenUtils;

pub fn check_nonnegative_amount(amount: i128) {
    if amount < 0 {
        panic!("negative amount is not allowed: {}", amount)
    }
}

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_LIFETIME_THRESHOLD, INSTANCE_BUMP_AMOUNT);
}

pub fn extend_balance_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, BALANCE_LIFETIME_THRESHOLD, BALANCE_BUMP_AMOUNT);
}

pub fn admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).unwrap()
}

pub fn read_allowance(env: &Env, from: Address, spender: Address) -> AllowanceValue {
    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    env.storage()
        .temporary()
        .get::<_, AllowanceValue>(&key)
        .map_or(
            AllowanceValue {
                amount: 0,
                expiration_ledger: 0,
            },
            |allowance| {
                if allowance.expiration_ledger < env.ledger().sequence() {
                    AllowanceValue {
                        amount: 0,
                        expiration_ledger: allowance.expiration_ledger,
                    }
                } else {
                    allowance
                }
            },
        )
}

pub fn write_allowance(
    env: &Env,
    from: Address,
    spender: Address,
    amount: i128,
    expiration_ledger: u32,
) {
    let allowance = AllowanceValue {
        amount,
        expiration_ledger,
    };

    if amount > 0 && expiration_ledger < env.ledger().sequence() {
        panic!("expiration_ledger is less than ledger seq when amount > 0")
    }

    let key = DataKey::Allowance(AllowanceDataKey { from, spender });
    env.storage().temporary().set(&key, &allowance);

    if amount > 0 {
        let live_for = expiration_ledger
            .checked_sub(env.ledger().sequence())
            .unwrap();

        env.storage()
            .temporary()
            .extend_ttl(&key, live_for, live_for)
    }
}

pub fn spend_allowance(env: &Env, from: Address, spender: Address, amount: i128) {
    let allowance = read_allowance(env, from.clone(), spender.clone());
    if allowance.amount < amount {
        panic!("insufficient allowance");
    }
    if amount > 0 {
        write_allowance(
            env,
            from,
            spender,
            allowance.amount - amount,
            allowance.expiration_ledger,
        );
    }
}

pub fn read_balance(env: &Env, addr: Address) -> i128 {
    let key = DataKey::Balance(addr);
    env.storage()
        .persistent()
        .get::<DataKey, i128>(&key)
        .map_or(0, |balance| {
            extend_balance_ttl(env, &key);
            balance
        })
}

fn write_balance(env: &Env, addr: Address, amount: i128) {
    let key = DataKey::Balance(addr);
    env.storage().persistent().set(&key, &amount);
    extend_balance_ttl(env, &key);
}

pub fn receive_balance(env: &Env, addr: Address, amount: i128) {
    let balance = read_balance(env, addr.clone());
    write_balance(env, addr, balance + amount);
}

pub fn spend_balance(env: &Env, addr: Address, amount: i128) {
    let balance = read_balance(env, addr.clone());
    if balance < amount {
        panic!("insufficient balance");
    }
    write_balance(env, addr, balance - amount);
}

pub fn read_decimal(env: &Env) -> u32 {
    TokenUtils::new(env).metadata().get_metadata().decimal
}

pub fn read_name(env: &Env) -> String {
    TokenUtils::new(env).metadata().get_metadata().name
}

pub fn read_symbol(env: &Env) -> String {
    TokenUtils::new(env).metadata().get_metadata().symbol
}

pub fn write_metadata(env: &Env, metadata: TokenMetadata) {
    TokenUtils::new(env).metadata().set_metadata(&metadata);
}
