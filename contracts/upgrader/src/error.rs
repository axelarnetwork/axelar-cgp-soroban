use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
pub enum ContractError {
    SameVersion = 1,
    VersionMismatch = 2,
}
