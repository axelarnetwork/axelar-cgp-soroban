use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Type};

pub fn axelar_executable(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let error_type = error_type(input)?;

    Ok(quote! {
        use stellar_axelar_gateway::executable::AxelarExecutableInterface as _;

        impl stellar_axelar_gateway::executable::DeriveOnly for #name {}

        #[stellar_axelar_std::contractimpl]
        impl AxelarExecutableInterface for #name {
            #[allow_during_migration]
            fn gateway(env: &Env) -> Address {
                Self::__gateway(env)
            }

            fn execute(
                env: &Env,
                source_chain: String,
                message_id: String,
                source_address: String,
                payload: Bytes,
            ) -> Result<(), #error_type> {
                stellar_axelar_gateway::executable::validate_message::<Self>(env, &source_chain, &message_id, &source_address, &payload).map_err(|err| match err {
                    stellar_axelar_gateway::executable::ValidationError::NotApproved => #error_type::NotApproved,
                })?;

                Self::__execute(env, source_chain, message_id, source_address, payload)
            }
        }
    })
}

fn error_type(input: &DeriveInput) -> syn::Result<Type> {
    let mut error_type = None;

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("axelar_executable"))
    {
        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("error") {
                return Err(meta.error("unsupported axelar_executable attribute"));
            }

            if error_type.is_some() {
                return Err(Error::new_spanned(
                    &meta.path,
                    "axelar_executable error type can only be specified once",
                ));
            }

            let value = meta.value()?;
            error_type = Some(value.parse()?);
            Ok(())
        })?;
    }

    error_type.ok_or_else(|| {
        Error::new_spanned(
            input,
            "AxelarExecutable requires #[axelar_executable(error = ContractError)]",
        )
    })
}

#[cfg(test)]
mod tests {
    fn formatted_axelar_executable_impl(contract_input: syn::DeriveInput) -> String {
        let axelar_executable_impl: proc_macro2::TokenStream =
            crate::axelar_executable::axelar_executable(&contract_input).unwrap();
        let axelar_executable_impl_file: syn::File = syn::parse2(axelar_executable_impl).unwrap();

        prettyplease::unparse(&axelar_executable_impl_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]")
    }

    #[test]
    fn axelar_executable_impl_generation_uses_configured_error_type() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(AxelarExecutable)]
            #[axelar_executable(error = ContractError)]
            pub struct Contract;
        };

        goldie::assert!(formatted_axelar_executable_impl(contract_input));
    }

    #[test]
    fn axelar_executable_impl_generation_requires_error_type() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(AxelarExecutable)]
            pub struct Contract;
        };

        let err = crate::axelar_executable::axelar_executable(&contract_input).unwrap_err();

        assert_eq!(
            err.to_string(),
            "AxelarExecutable requires #[axelar_executable(error = ContractError)]"
        );
    }

    #[test]
    fn axelar_executable_impl_generation_rejects_unsupported_attribute() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(AxelarExecutable)]
            #[axelar_executable(unsupported = Foo)]
            pub struct Contract;
        };

        let err = crate::axelar_executable::axelar_executable(&contract_input).unwrap_err();

        assert_eq!(err.to_string(), "unsupported axelar_executable attribute");
    }

    #[test]
    fn axelar_executable_impl_generation_rejects_duplicate_error_type() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(AxelarExecutable)]
            #[axelar_executable(error = ContractError, error = OtherError)]
            pub struct Contract;
        };

        let err = crate::axelar_executable::axelar_executable(&contract_input).unwrap_err();

        assert_eq!(
            err.to_string(),
            "axelar_executable error type can only be specified once"
        );
    }
}
