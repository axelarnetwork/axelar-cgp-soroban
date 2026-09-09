use proc_macro2::{Ident, TokenStream as TokenStream2, TokenStream};
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn};

use crate::utils::{parse_env_identifier, PrependStatement};

pub fn pausable(name: &Ident) -> TokenStream2 {
    quote! {
        use stellar_axelar_std::interfaces::PausableInterface as _;

        #[stellar_axelar_std::contractimpl]
        impl stellar_axelar_std::interfaces::PausableInterface for #name {
            fn paused(env: &Env) -> bool {
                stellar_axelar_std::interfaces::paused(env)
            }

            fn pause(env: &Env) {
                stellar_axelar_std::interfaces::pause::<Self, _>(env, ContractError::AlreadyPaused);
            }

            fn unpause(env: &Env) {
                stellar_axelar_std::interfaces::unpause::<Self, _>(env, ContractError::NotPaused);
            }
        }
    }
}

pub fn when_not_paused_impl(mut input_fn: ItemFn) -> Result<TokenStream, syn::Error> {
    let env_ident = parse_env_identifier(&input_fn.sig.inputs)?;

    let pause_stmt = parse_quote! {
        stellar_axelar_std::ensure!(!Self::paused(#env_ident), ContractError::ContractPaused);
    };

    Ok(input_fn.prepend_statement(pause_stmt).into_token_stream())
}
