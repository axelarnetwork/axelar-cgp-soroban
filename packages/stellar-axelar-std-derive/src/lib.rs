//! Note: The tests are located in the `stellar-axelar-std` package instead of `stellar-axelar-std-derive`
//!
//! This ensures compatibility and prevents cyclic dependency issues during testing and release.

mod axelar_executable;
mod contractimpl;
mod contractstorage;
mod into_event;
mod its_executable;
mod operatable;
mod ownable;
mod pausable;
mod upgradable;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemImpl};

/// Designates functions in an `impl` block as contract entrypoints.
///
/// This is a wrapper around the soroban-sdk's `#[contractimpl]` attribute.
/// It adds additional checks to ensure entrypoints don't get accidentally, or maliciously, called
/// after a contract upgrade, but before the data migration is complete.
///
/// # Example
/// ```rust, ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contracterror};
/// use stellar_axelar_std_derive::{contractimpl, Upgradable};
///
/// #[contract]
/// #[derive(Upgradable)]
/// pub struct Contract;
///
/// // any function in this impl block will panic if called during migration
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env) {
///         // constructor code
///     }
///
///     pub fn do_something(env: &Env, arg: String) {
///         // entrypoint code
///     }
/// }
///
/// #[contracterror]
/// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// #[repr(u32)]
/// pub enum ContractError {
///     MigrationInProgress = 1,
/// }
///
/// // if an entrypoint is able to return a Result<_, ContractError>,
/// // it will return ContractError::MigrationInProgress instead of panicking when called during migration
/// #[contractimpl]
/// impl Contract {
///     pub fn return_result(env: &Env, arg: String) -> Result<u32, ContractError> {
///         // entrypoint code
///     }
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn contractimpl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemImpl);

    contractimpl::contractimpl(&mut input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Implements the Operatable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Operatable;
///
/// #[contract]
/// #[derive(Operatable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_operator(env, &owner);
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Operatable)]
pub fn derive_operatable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    operatable::operatable(name).into()
}

/// Implements the Ownable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Ownable;
///
/// #[contract]
/// #[derive(Ownable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_owner(env, &owner);
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Ownable)]
pub fn derive_ownable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    ownable::ownable(name).into()
}

/// Implements the Pausable interface for a Soroban contract.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std_derive::Pausable;
///
/// #[contract]
/// #[derive(Pausable)]
/// pub struct Contract;
/// # }
/// ```
#[proc_macro_derive(Pausable)]
pub fn derive_pausable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    pausable::pausable(name).into()
}

/// Ensure that the Stellar contract is not paused before executing the function.
///
/// The first argument to the function must be `env`, and a `ContractError` error type must be defined in scope,
/// with a `ContractPaused` variant.
///
/// # Example
/// ```rust,ignore
/// # use stellar_axelar_std::{contract, contractimpl, contracttype, Address, Env};
/// use stellar_axelar_std::{Pausable, when_not_paused};
///
/// #[contracttype]
/// pub enum ContractError {
///     ContractPaused = 1,
/// }
///
/// #[contract]
/// #[derive(Pausable)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[when_not_paused]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn when_not_paused(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    pausable::when_not_paused_impl(input_fn)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Implements the Upgradable and Migratable interfaces for a Soroban contract.
///
/// A `ContractError` error type must be defined in scope, and have a `MigrationNotAllowed` variant.
/// A default migration implementation is automatically provided. If custom migration code is required,
/// the `#[migratable]` attribute can be applied to the contract struct.
/// It defaults to unit migration data. Use `#[migratable(data = MigrationData)]`
/// if the migration needs a custom input type.
/// In that case, the contract must implement the `CustomMigratableInterface` trait. The associated `Error` type
/// must implement the `Into<ContractError>` trait. The `ContractError` type itself implements it implicitly,
/// so that is an easy way to use it.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contractimpl, contracterror, Address, Env};
/// use stellar_axelar_std_derive::{Ownable, Upgradable};
/// # #[contracterror]
/// # #[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
/// # #[repr(u32)]
/// # pub enum ContractError {
/// #     MigrationNotAllowed = 1,
/// # }
///
/// #[contract]
/// #[derive(Ownable, Upgradable)]
/// #[migratable(data = Address)]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(env: &Env, owner: Address) {
///         stellar_axelar_std::interfaces::set_owner(env, &owner);
///     }
/// }
///
/// impl CustomMigratableInterface for Contract {
///     type MigrationData = Address;
///     type Error = ContractError;
///
///     fn __migrate(env: &Env, new_owner: Self::MigrationData) -> Result<(), Self::Error> {
///         Self::transfer_ownership(env, new_owner);
///         Ok(())
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(Upgradable, attributes(migratable))]
pub fn derive_upgradable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    upgradable::upgradable(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Implements the Event trait for a Stellar contract event.
///
/// Fields without a `#[data]` attribute are used as topics, while fields with `#[data]` are used as event data.
/// The event name can be specified with `#[event_name(...)]` or will default to the struct name in snake_case (minus "Event" suffix).
///
/// # Data payload encoding
///
/// `#[data]` publishes the data payload as a `Vec<Val>`, even when there is only one such field.
/// `#[datum]` publishes a single field as a bare `Val` instead of a one-element `Vec`. Pick one
/// encoding per event; the derive rejects at compile time:
/// - a tuple struct, since unnamed fields cannot be emitted;
/// - a field carrying more than one `#[data]`/`#[datum]` attribute;
/// - more than one `#[datum]` field, or a `#[datum]` combined with any `#[data]` field, since only
///   the first data field would be published.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// use core::fmt::Debug;
/// use stellar_axelar_std::events::Event;
/// use stellar_axelar_std::IntoEvent;
/// use stellar_axelar_std::{Address, contract, contractimpl, Env, String};
///
/// #[derive(Debug, PartialEq, IntoEvent)]
/// #[event_name("transfer")]
/// pub struct TransferEvent {
///     pub from: Address,
///     pub to: Address,
///     #[data]
///     pub amount: String,
/// }
///
/// #[contract]
/// pub struct Token;
///
/// #[contractimpl]
/// impl Token {
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///
///         // Generates event with:
///         // - Topics: ["transfer", contract_address, to]
///         // - Data: [amount]
///         TransferEvent {
///             from: env.current_contract_address(),
///             to,
///             amount,
///         }.emit(env);
///     }
/// }
/// }
/// ```
#[proc_macro_derive(IntoEvent, attributes(event_name, datum, data))]
pub fn derive_into_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    into_event::into_event(&input).into()
}

#[proc_macro_derive(InterchainTokenExecutable)]
pub fn derive_its_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    its_executable::its_executable(name).into()
}

/// Implements the Axelar Executable interface for a Soroban contract.
///
/// The concrete error type must be specified with `#[axelar_executable(error = ...)]`.
/// It must match the contract's `CustomAxelarExecutable::Error` associated type and
/// define a `NotApproved` variant used when the gateway has not approved the message.
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// # use stellar_axelar_std::{contract, contracterror, Address, Bytes, Env, String};
/// use stellar_axelar_std_derive::AxelarExecutable;
/// use stellar_axelar_gateway::executable::CustomAxelarExecutable;
///
/// #[contracterror]
/// #[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// #[repr(u32)]
/// pub enum ContractError {
///     NotApproved = 1,
/// }
///
/// #[contract]
/// #[derive(AxelarExecutable)]
/// #[axelar_executable(error = ContractError)]
/// pub struct Contract;
///
/// impl CustomAxelarExecutable for Contract {
///     type Error = ContractError;
///
///     fn __gateway(env: &Env) -> Address {
///         todo!()
///     }
///
///     fn __execute(
///         env: &Env,
///         source_chain: String,
///         message_id: String,
///         source_address: String,
///         payload: Bytes,
///     ) -> Result<(), Self::Error> {
///         Ok(())
///     }
/// }
/// # }
/// ```
#[proc_macro_derive(AxelarExecutable, attributes(axelar_executable))]
pub fn derive_axelar_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    axelar_executable::axelar_executable(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Ensures that only a contract's owner can execute the attributed function.
///
/// The first argument to the function must be `env`
///
/// # Example
/// ```rust,ignore
/// # use stellar_axelar_std::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std::only_owner;
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[only_owner]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn only_owner(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    ownable::only_owner_impl(input_fn)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Ensures that only a contract's operator can execute the attributed function.
///
/// The first argument to the function must be `env`
///
/// # Example
/// ```rust,ignore
/// # use stellar_axelar_std::{contract, contractimpl, Address, Env};
/// use stellar_axelar_std::only_operator;
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     #[only_operator]
///     pub fn transfer(env: &Env, to: Address, amount: String) {
///         // ... transfer logic ...
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn only_operator(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    operatable::only_operator_impl(input_fn)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Implements a storage interface for a Stellar contract storage enum.
///
/// The enum variants define contract data keys, with optional named fields as contract data map keys.
/// Each variant requires a `#[value(Type)]` xor `#[status]` attribute to specify the stored value type.
/// Storage type can be specified with `#[instance]`, `#[persistent]`, or `#[temporary]` attributes (defaults to instance).
///
/// Certain types have default behaviors for TTL extensions:
/// - `#[persistent]`: This is extended by default every time a data key is accessed, for that data key.
///   The persistent data type does not share the same TTL as the contract instance.
/// - `#[instance]`: This is extended by default for all contract endpoints, so it does not need to be included in generated data key access functions.
///   This also serves to extend the lifetime of the contract's bytecode, since the instance data type does share the same TTL as the contract instance.
/// - `#[temporary]`: This is not extended by default, since this data type can be easily recreated or only valid for a certain period of time.
///   In the special case that temporary data needs to be extended, a user may call the generated #ttl_extender function for that temporary data key.
///
/// More on Stellar data types: <https://developers.stellar.org/docs/learn/encyclopedia/storage/state-archival#contract-data-type-descriptions>
///
/// # Example
/// ```rust,ignore
/// # mod test {
/// use stellar_axelar_std::{contract, contractimpl, contractype, Address, Env, String};
/// use stellar_axelar_std::contractstorage;
///
/// #[contractstorage]
/// #[derive(Clone, Debug)]
/// enum DataKey {
///     #[instance]
///     #[value(Address)]
///     Owner,
///
///     #[persistent]
///     #[value(String)]
///     TokenName { token_id: u32 },
///
///     #[temporary]
///     #[value(u64)]
///     LastUpdate { account: Address },
///
///     #[instance]
///     #[status]
///     Paused,
/// }
///
/// #[contract]
/// pub struct Contract;
///
/// #[contractimpl]
/// impl Contract {
///     pub fn __constructor(
///         env: &Env,
///         token_id: u32,
///         name: String,
///     ) {
///         storage::set_token_name(env, token_id, &name);
///     }
///
///     pub fn foo(env: &Env, token_id: u32) -> Option<String> {
///         storage::token_name(env, token_id);
///     }
///
///     pub fn bar(env: &Env, token_id: u32) -> Option<String> {
///         storage::remove_token_name(env, token_id)
///     }
/// }
/// # }
/// ```
#[proc_macro_attribute]
pub fn contractstorage(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);

    contractstorage::contract_storage(&input).into()
}
