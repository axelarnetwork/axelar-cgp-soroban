#![no_std]
use soroban_sdk::{contract, contractimpl, log, symbol_short, Address, Env, IntoVal, Symbol, Val, Vec};

const COUNTER: Symbol = symbol_short!("COUNTER");

#[contract]
pub struct IncrementorContract;

#[contractimpl]
impl IncrementorContract {
    /// Increment an internal counter; return the new value.
    pub fn increment(env: Env) -> u32 {
        let mut count: u32 = env.storage().instance().get(&COUNTER).unwrap_or(0);

        count += 1;

        log!(&env, "count: {}", count);

        env.storage().instance().set(&COUNTER, &count);

        env.storage().instance().extend_ttl(100, 100);

        count
    }
}

#[contract]
pub struct Incrementor2Contract;

#[contractimpl]
impl Incrementor2Contract {
    pub fn increment_twice(env: Env, contract_id: Address) -> u32 {
        let _: u32 = env.invoke_contract(&contract_id, &Symbol::new(&env, "increment"), Vec::<Val>::new(&env).into_val(&env));

        log!(env, "incremented once");

        let count: u32 = env.invoke_contract(&contract_id, &Symbol::new(&env, "increment"), Vec::<Val>::new(&env).into_val(&env));

        log!(env, "incremented twice");

        count
    }
}

#[cfg(test)]
mod test;
