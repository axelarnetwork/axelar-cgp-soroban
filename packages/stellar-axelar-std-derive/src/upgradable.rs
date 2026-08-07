use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;
use syn::{DeriveInput, Error, Type};

pub fn upgradable(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let migration_data_type = migration_data_type(input)?;
    let (custom_migration_impl, migration_data_type) = migration_data_type.map_or_else(
        || (default_custom_migration(name), quote! { () }),
        |migration_data_type| (quote! {}, quote! { #migration_data_type }),
    );

    Ok(quote! {
        use stellar_axelar_std::interfaces::{UpgradableInterface as _, MigratableInterface as _};

        #[stellar_axelar_std::contractimpl]
        impl stellar_axelar_std::interfaces::UpgradableInterface for #name {
            #[allow_during_migration]
            fn version(env: &Env) -> stellar_axelar_std::String {
                stellar_axelar_std::String::from_str(env, env!("CARGO_PKG_VERSION"))
            }

            #[allow_during_migration]
            fn required_auths(env: &Env) -> stellar_axelar_std::Vec<stellar_axelar_std::Address> {
                stellar_axelar_std::interfaces::required_auths::<Self>(env)
            }

            // if upgrade is not allowed during migration, the contract could get completely bricked
            // if there is a bug in the migration code and the contract can't be upgraded again
            #[allow_during_migration]
            fn upgrade(env: &Env, new_wasm_hash: stellar_axelar_std::BytesN<32>) {
                stellar_axelar_std::interfaces::upgrade::<Self>(env, new_wasm_hash);
            }
        }

        #[stellar_axelar_std::contractimpl]
        impl stellar_axelar_std::interfaces::MigratableInterface for #name {
            type Error = ContractError;

            #[allow_during_migration]
            fn migrate(env: &Env, migration_data: #migration_data_type) -> Result<(), ContractError> {
                stellar_axelar_std::interfaces::migrate::<Self>(env, migration_data)
                    .map_err(|err| match err {
                        stellar_axelar_std::interfaces::MigrationError::NotAllowed => ContractError::MigrationNotAllowed,
                        stellar_axelar_std::interfaces::MigrationError::ExecutionFailed(err) => err.into(),
                    }
                )
            }
        }

        #custom_migration_impl
    })
}

fn default_custom_migration(name: &Ident) -> TokenStream2 {
    quote! {
        impl stellar_axelar_std::interfaces::CustomMigratableInterface for #name {
            type MigrationData = ();
            type Error = ContractError;

            fn __migrate(_env: &Env, _migration_data: Self::MigrationData) -> Result<(), Self::Error> {
                Ok(())
            }
        }
    }
}

fn migration_data_type(input: &DeriveInput) -> syn::Result<Option<Type>> {
    let mut migration_data_type = None;
    let mut has_migratable_attr = false;

    for attr in input
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("migratable"))
    {
        if has_migratable_attr {
            return Err(Error::new_spanned(
                attr,
                "migratable attribute can only be specified once",
            ));
        }

        has_migratable_attr = true;

        if attr.meta.require_path_only().is_ok() {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if !meta.path.is_ident("data") {
                return Err(meta.error("unsupported migratable attribute"));
            }

            if migration_data_type.is_some() {
                return Err(Error::new_spanned(
                    &meta.path,
                    "migration data type can only be specified once",
                ));
            }

            let value = meta.value()?;
            migration_data_type = Some(value.parse()?);
            Ok(())
        })?;
    }

    if !has_migratable_attr {
        return Ok(None);
    }

    Ok(Some(
        migration_data_type.unwrap_or_else(|| syn::parse_quote! { () }),
    ))
}

/// Tests the upgradable impl generation for a contract.
#[cfg(test)]
mod tests {

    #[test]
    fn upgradable_impl_generation_succeeds() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable(data = MigrationData)]
            pub struct Contract;
        };

        let upgradable_impl: proc_macro2::TokenStream =
            crate::upgradable::upgradable(&contract_input).unwrap();
        let upgradable_impl_file: syn::File = syn::parse2(upgradable_impl).unwrap();
        let formatted_upgradable_impl = prettyplease::unparse(&upgradable_impl_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");

        goldie::assert!(formatted_upgradable_impl);
    }

    #[test]
    fn default_upgradable_impl_generation_uses_unit_migration_data() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            pub struct Contract;
        };

        let upgradable_impl: proc_macro2::TokenStream =
            crate::upgradable::upgradable(&contract_input).unwrap();
        let upgradable_impl_file: syn::File = syn::parse2(upgradable_impl).unwrap();
        let formatted_upgradable_impl = prettyplease::unparse(&upgradable_impl_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");

        goldie::assert!(formatted_upgradable_impl);
    }

    #[test]
    fn custom_upgradable_impl_generation_defaults_to_unit_migration_data() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable]
            pub struct Contract;
        };

        let upgradable_impl: proc_macro2::TokenStream =
            crate::upgradable::upgradable(&contract_input).unwrap();
        let upgradable_impl_file: syn::File = syn::parse2(upgradable_impl).unwrap();
        let formatted_upgradable_impl = prettyplease::unparse(&upgradable_impl_file)
            .replace("pub fn ", "\npub fn ")
            .replace("#[cfg(test)]", "\n#[cfg(test)]");

        goldie::assert!(formatted_upgradable_impl);
    }

    #[test]
    fn upgradable_impl_generation_rejects_duplicate_migratable_attribute() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable]
            #[migratable]
            pub struct Contract;
        };

        let err = crate::upgradable::upgradable(&contract_input).unwrap_err();

        assert_eq!(
            err.to_string(),
            "migratable attribute can only be specified once"
        );
    }

    #[test]
    fn upgradable_impl_generation_rejects_unsupported_migratable_attribute() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable(unsupported = Foo)]
            pub struct Contract;
        };

        let err = crate::upgradable::upgradable(&contract_input).unwrap_err();

        assert_eq!(err.to_string(), "unsupported migratable attribute");
    }

    #[test]
    fn upgradable_impl_generation_rejects_duplicate_migration_data_type() {
        let contract_input: syn::DeriveInput = syn::parse_quote! {
            #[contract]
            #[derive(Ownable, Upgradable)]
            #[migratable(data = MigrationData, data = OtherData)]
            pub struct Contract;
        };

        let err = crate::upgradable::upgradable(&contract_input).unwrap_err();

        assert_eq!(
            err.to_string(),
            "migration data type can only be specified once"
        );
    }
}
