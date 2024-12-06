mod operatable;
mod ownable;
#[cfg(test)]
mod testdata;
mod upgradable;

pub use operatable::*;
pub use ownable::*;
pub use upgradable::*;

/// This submodule encapsulates data keys for the separate interfaces. These keys break naming conventions on purpose.
/// If a contract implements a contract type that would result in a collision with a key defined here,
/// the linter will complain about it. So as long as contracts follow regular naming conventions,
/// there is no risk of collisions.
mod storage {
    #![allow(non_camel_case_types)]

    use soroban_sdk::contracttype;

    // add a separate data key type for each interface. Using a single enum could lead to unintentionally breaks
    // of unrelated interfaces, because the key serialization is variant order dependent.

    #[contracttype]
    pub enum OperatorDataKey {
        Interfaces_Operator,
    }

    #[contracttype]
    pub enum OwnerDataKey {
        Interfaces_Owner,
    }

    #[contracttype]
    pub enum MigratingDataKey {
        Interfaces_Migrating,
    }
}
