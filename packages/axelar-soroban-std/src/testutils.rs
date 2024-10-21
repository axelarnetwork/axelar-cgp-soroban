#![cfg(any(test, feature = "testutils"))]
extern crate std;

use soroban_sdk::{
    testutils::{AuthorizedFunction, AuthorizedInvocation, Events},
    vec, Address, Env, IntoVal, Symbol, Val, Vec,
};

/// Asserts invocation auth of a contract from a single caller.
pub fn assert_invocation<T>(
    env: &Env,
    caller: &Address,
    contract_id: &Address,
    function_name: &str,
    args: T,
) where
    T: IntoVal<Env, Vec<Val>>,
{
    assert_eq!(
        env.auths(),
        std::vec![(
            caller.clone(),
            AuthorizedInvocation {
                function: AuthorizedFunction::Contract((
                    contract_id.clone(),
                    Symbol::new(env, function_name),
                    args.into_val(env),
                )),
                sub_invocations: std::vec![]
            }
        )]
    );
}

/// Asserts that the event at `event_index` in the environment's emitted events is the expected event.
/// If `event_index` is negative, the length of events will be added to it, i.e it'll be indexed from the end.
pub fn assert_emitted_event<U, V>(
    env: &Env,
    mut event_index: i32,
    contract_id: &Address,
    topics: U,
    data: V,
) where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    let events = env.events().all();
    if event_index.is_negative() {
        event_index += events.len() as i32;
    }

    assert!(
        event_index < events.len() as i32,
        "event {} not found, only {} events were emitted",
        event_index + 1,
        events.len()
    );

    let event = events.get(event_index as u32).unwrap();

    assert_eq!(event.0, contract_id.clone());
    assert_eq!(event.1, topics.into_val(env));
    assert_eq!(vec![env, event.2], vec![env, data.into_val(env)]);
}

pub fn assert_last_emitted_event<U, V>(env: &Env, contract_id: &Address, topics: U, data: V)
where
    U: IntoVal<Env, Vec<Val>>,
    V: IntoVal<Env, Val>,
{
    assert_emitted_event(env, -1, contract_id, topics, data);
}
