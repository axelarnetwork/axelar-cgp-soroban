use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_attribute]
pub fn ownable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
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

#[proc_macro_attribute]
pub fn upgradable(_attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
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
    };

    TokenStream::from(expanded)
}
