use core::fmt::Debug;

use soroban_sdk::{assert_with_error, contractclient, Env};

use crate as stellar_axelar_std;
use crate::events::Event;
use crate::interfaces::{storage, OwnableInterface};
use crate::IntoEvent;

#[contractclient(name = "PausableClient")]
pub trait PausableInterface: OwnableInterface {
    /// Returns whether the contract is currently paused.
    fn paused(env: &Env) -> bool;

    /// Pauses the contract. Only callable by the owner.
    fn pause(env: &Env);

    /// Unpauses the contract. Only callable by the owner.
    fn unpause(env: &Env);
}

/// Default implementation of the [`PausableInterface`] trait.
pub fn paused(env: &Env) -> bool {
    storage::pausable::is_interfaces_paused(env)
}

/// Default implementation of the [`PausableInterface`] trait.
///
/// Panics with `already_paused` if the contract is already paused, so that a redundant call
/// surfaces the likely configuration mistake instead of re-emitting a [`PausedEvent`] that
/// reflects no state change.
pub fn pause<T: PausableInterface, E: Into<soroban_sdk::Error>>(env: &Env, already_paused: E) {
    T::owner(env).require_auth();

    assert_with_error!(env, !paused(env), already_paused);

    storage::pausable::set_interfaces_paused_status(env);

    PausedEvent {}.emit(env);
}

/// Default implementation of the [`PausableInterface`] trait.
///
/// Panics with `not_paused` if the contract is not currently paused, so that a redundant call
/// surfaces the likely configuration mistake instead of re-emitting an [`UnpausedEvent`] that
/// reflects no state change.
pub fn unpause<T: PausableInterface, E: Into<soroban_sdk::Error>>(env: &Env, not_paused: E) {
    T::owner(env).require_auth();

    assert_with_error!(env, paused(env), not_paused);

    storage::pausable::remove_interfaces_paused_status(env);

    UnpausedEvent {}.emit(env);
}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct PausedEvent {}

#[derive(Clone, Debug, PartialEq, Eq, IntoEvent)]
pub struct UnpausedEvent {}

#[cfg(test)]
mod test {
    use soroban_sdk::Error;
    use stellar_axelar_std::testutils::Address as _;
    use stellar_axelar_std::{contract, contracterror, Address, Env};
    use stellar_axelar_std_derive::contractimpl;

    use super::{PausedEvent, UnpausedEvent};
    use crate as stellar_axelar_std;
    use crate::interfaces::{OwnableInterface, PausableInterface};
    use crate::{assert_auth, assert_auth_err, assert_contract_err, events, when_not_paused};

    #[contracterror]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[repr(u32)]
    pub enum ContractError {
        ContractPaused = 1,
        MigrationInProgress = 2,
        AlreadyPaused = 3,
        NotPaused = 4,
    }

    #[contract]
    pub struct Contract;

    #[contractimpl]
    impl Contract {
        pub fn __constructor(env: &Env, owner: Address) {
            crate::interfaces::ownable::set_owner(env, &owner);
        }
    }

    #[contractimpl]
    impl PausableInterface for Contract {
        fn paused(env: &Env) -> bool {
            super::paused(env)
        }

        fn pause(env: &Env) {
            super::pause::<Self, _>(env, ContractError::AlreadyPaused)
        }

        fn unpause(env: &Env) {
            super::unpause::<Self, _>(env, ContractError::NotPaused)
        }
    }

    #[contractimpl]
    impl OwnableInterface for Contract {
        fn owner(env: &Env) -> Address {
            crate::interfaces::ownable::owner(env)
        }

        fn transfer_ownership(env: &Env, new_owner: Address) {
            crate::interfaces::ownable::transfer_ownership::<Self>(env, new_owner)
        }
    }

    #[contractimpl]
    impl Contract {
        #[when_not_paused]
        pub fn test(env: &Env) -> Result<u32, ContractError> {
            Ok(42)
        }
    }

    fn setup<'a>() -> (Env, ContractClient<'a>) {
        let env = Env::default();
        let owner = Address::generate(&env);
        let contract_id = env.register(Contract, (owner,));
        let client = ContractClient::new(&env, &contract_id);
        (env, client)
    }

    #[test]
    fn paused_returns_false_by_default() {
        let (_, client) = setup();
        assert!(!client.paused());
    }

    #[test]
    fn pause_succeeds_when_not_paused() {
        let (env, client) = setup();

        assert!(!client.paused());

        assert_auth!(client.owner(), client.pause());
        goldie::assert!(events::fmt_last_emitted_event::<PausedEvent>(&env));

        assert!(client.paused());
    }

    #[test]
    fn pause_fails_when_already_paused() {
        let (_, client) = setup();

        assert_auth!(client.owner(), client.pause());
        assert!(client.paused());

        assert_contract_err!(
            client.mock_all_auths().try_pause(),
            Error::from(ContractError::AlreadyPaused)
        );
        assert!(client.paused());
    }

    #[test]
    fn pause_fails_when_not_owner() {
        let (env, client) = setup();

        assert_auth_err!(Address::generate(&env), client.pause());
    }

    #[test]
    fn unpause_succeeds_when_paused() {
        let (env, client) = setup();

        assert_auth!(client.owner(), client.pause());
        assert!(client.paused());

        assert_auth!(client.owner(), client.unpause());
        goldie::assert!(events::fmt_last_emitted_event::<UnpausedEvent>(&env));

        assert!(!client.paused());
    }

    #[test]
    fn unpause_fails_when_not_paused() {
        let (_, client) = setup();

        assert!(!client.paused());

        assert_contract_err!(
            client.mock_all_auths().try_unpause(),
            Error::from(ContractError::NotPaused)
        );
        assert!(!client.paused());
    }

    #[test]
    fn unpause_fails_when_not_owner() {
        let (env, client) = setup();

        assert_auth_err!(Address::generate(&env), client.unpause());
    }

    #[test]
    fn test_succeeds_when_not_paused() {
        let (_, client) = setup();
        assert_eq!(client.test(), 42);
    }

    #[test]
    fn test_fails_when_paused() {
        let (_, client) = setup();

        client.mock_all_auths().pause();
        assert_contract_err!(client.try_test(), ContractError::ContractPaused);
    }
}
