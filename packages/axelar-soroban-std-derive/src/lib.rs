use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::Parse, parse::ParseStream, parse_macro_input,
    DeriveInput, Error, Ident, Token, Type,
};

#[proc_macro_attribute]
pub fn ownable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        use ::axelar_soroban_std::interfaces::OwnableInterface;

        #input

        #[::soroban_sdk::contractimpl]
        impl ::axelar_soroban_std::interfaces::OwnableInterface for #name {
            fn owner(env: &Env) -> ::soroban_sdk::Address {
                ::axelar_soroban_std::interfaces::owner(env)
            }

            fn transfer_ownership(env: &Env, new_owner: ::soroban_sdk::Address) {
                ::axelar_soroban_std::interfaces::transfer_ownership::<Self>(env, new_owner);
            }
        }
    };

    TokenStream::from(expanded)
}

struct UpgradableArgs {
    migration_data: Option<Type>,
    ownable_impl: bool,
}

impl Parse for UpgradableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                migration_data: None,
                ownable_impl: true,
            });
        }

        let mut migration_data = None;
        let mut ownable_impl = true;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "migration_data" => {
                    migration_data = Some(input.parse::<Type>()?);
                }
                "ownable_impl" => {
                    ownable_impl = input.parse::<syn::LitBool>()?.value;
                }
                _ => return Err(Error::new(ident.span(), "expected `migration_data=..` or `ownable_impl=..`")),
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            migration_data,
            ownable_impl,
        })
    }
}

#[proc_macro_attribute]
pub fn upgradable(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UpgradableArgs);
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let migration_data = match &args.migration_data {
        Some(ty) => quote! { #ty },
        None => quote! { () },
    };

    let ownable_impl = if args.ownable_impl {
        quote! { #[::axelar_soroban_std_derive::ownable] }
    } else {
        quote! {}
    };

    let expanded = quote! {
        use ::axelar_soroban_std::interfaces::{UpgradableInterface, MigratableInterface};

        #ownable_impl
        #input

        #[::soroban_sdk::contractimpl]
        impl ::axelar_soroban_std::interfaces::UpgradableInterface for #name {
            fn version(env: &Env) -> ::soroban_sdk::String {
                ::soroban_sdk::String::from_str(env, env!("CARGO_PKG_VERSION"))
            }
            fn upgrade(env: &Env, new_wasm_hash: ::soroban_sdk::BytesN<32>) {
                ::axelar_soroban_std::interfaces::upgrade::<Self>(env, new_wasm_hash);
            }
        }

        #[::soroban_sdk::contractimpl]
        impl ::axelar_soroban_std::interfaces::MigratableInterface for #name {
            type MigrationData = #migration_data;
            type Error = ContractError;

            fn migrate(env: &Env, migration_data: #migration_data) -> Result<(), ContractError> {
                ::axelar_soroban_std::interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
                    .map_err(|_| ContractError::MigrationNotAllowed)
            }
        }
    };
    TokenStream::from(expanded)
}
