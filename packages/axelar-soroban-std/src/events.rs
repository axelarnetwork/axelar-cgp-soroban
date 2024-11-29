use core::fmt::Debug;
use soroban_sdk::testutils::Events;
use soroban_sdk::{Address, Env, IntoVal, Topics, TryFromVal, Val, Vec};

pub trait Event: TryFromVal<Env, (Vec<Val>, Val)> + Debug + PartialEq {
    fn topic() -> impl Topics;
    fn data(&self) -> impl IntoVal<Env, Val>;

    fn emit(&self, env: &Env) {
        env.events().publish(Self::topic(), self.data());
    }
}

pub fn parse_last_emitted_event<E>(env: &Env) -> Result<(Address, E), Error>
where
    E: Event,
{
    env.events()
        .all()
        .last()
        .and_then(|event| convert_emitted_event(env, event))
        .ok_or(Error::EventNotFound)
}

pub fn parse_emitted_event_at_idx<E>(env: &Env, idx: u32) -> Result<(Address, E), Error>
where
    E: Event,
{
    env.events()
        .all()
        .get(idx)
        .and_then(|event| convert_emitted_event(env, event))
        .ok_or(Error::EventNotFound)
}

fn convert_emitted_event<E>(
    env: &Env,
    (contract_id, topics, data): (Address, Vec<Val>, Val),
) -> Option<(Address, E)>
where
    E: Event,
{
    E::try_from_val(env, &(topics, data))
        .ok()
        .map(|e| (contract_id, e))
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    EventNotFound,
}

#[cfg(test)]
mod test {}
