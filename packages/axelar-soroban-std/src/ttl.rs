use soroban_sdk::Env;

/// Parameters for extending the contract instance and its instance storage.
///
/// If the instance's time to live falls below 14 days, it will be extended by 60 days.
///
/// If at least one message is approved every 14 days, the instance should never be archived.
pub const LEDGERS_PER_DAY: u32 = (24 * 3600) / 5;
pub const INSTANCE_TTL_THRESHOLD: u32 = 14 * LEDGERS_PER_DAY;
pub const INSTANCE_TTL_EXTEND_TO: u32 = 60 * LEDGERS_PER_DAY;

pub fn extend_instance_ttl(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
}
