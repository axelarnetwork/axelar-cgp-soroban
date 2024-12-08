use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, parse::Parse, parse::ParseStream, Token, punctuated::Punctuated,
    Type, Ident, parenthesized, Error};


#[proc_macro_attribute]
pub fn ownable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        use axelar_soroban_std::interfaces::{OwnableInterface};

        #input

        #[soroban_sdk::contractimpl]
        impl axelar_soroban_std::interfaces::OwnableInterface for #name {
            fn owner(env: &Env) -> soroban_sdk::Address {
                axelar_soroban_std::interfaces::owner(env)
            }

            fn transfer_ownership(env: &Env, new_owner: soroban_sdk::Address) {
                axelar_soroban_std::interfaces::transfer_ownership::<Self>(env, new_owner);
            }
        }
    };

    TokenStream::from(expanded)
}

struct UpgradableArgs {
    migrate_types: Vec<Type>,
}

impl Parse for UpgradableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { migrate_types: vec![] });
        }

        let ident: Ident = input.parse()?;
        if ident != "migrate" {
            return Err(Error::new(ident.span(), "expected `migrate`"));
        }

        input.parse::<Token![=]>()?;
        let content;
        parenthesized!(content in input);

        let types = Punctuated::<Type, Token![,]>::parse_terminated(&content)?;
        Ok(Self {
            migrate_types: types.into_iter().collect(),
        })
    }
}
#[proc_macro_attribute]
pub fn upgradable(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as UpgradableArgs);
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let migration_args = if args.migrate_types.is_empty() {
        quote! { () }
    } else {
        let types = &args.migrate_types;
        quote! { (#(#types),*) }
    };

    let expanded = quote! {
        use axelar_soroban_std::interfaces::{UpgradableInterface, MigratableInterface};

        #[axelar_soroban_std_derive::ownable]
        #input

        #[soroban_sdk::contractimpl]
        impl axelar_soroban_std::interfaces::UpgradableInterface for #name {
            fn version(env: &Env) -> soroban_sdk::String {
                soroban_sdk::String::from_str(env, env!("CARGO_PKG_VERSION"))
            }

            fn upgrade(env: &Env, new_wasm_hash: soroban_sdk::BytesN<32>) {
                axelar_soroban_std::interfaces::upgrade::<Self>(env, new_wasm_hash);
            }
        }

        #[soroban_sdk::contractimpl]
        impl axelar_soroban_std::interfaces::MigratableInterface for #name {
            type MigrationData = #migration_args;
            type Error = ContractError;

            fn migrate(env: &Env, migration_data: #migration_args) -> Result<(), ContractError> {
                axelar_soroban_std::interfaces::migrate::<Self>(env, || Self::run_migration(env, migration_data))
                    .map_err(|_| ContractError::MigrationNotAllowed)
            }
        }
    };
    TokenStream::from(expanded)
}
