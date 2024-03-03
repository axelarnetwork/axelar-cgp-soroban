extern crate std;

use soroban_sdk::{Env, IntoVal, TryFromVal, Val, Vec};

pub trait IntoVec<T> {
    fn into_vec(self, env: &Env) -> Vec<T>;
}

impl<T: Clone + IntoVal<Env, Val> + TryFromVal<Env, Val>> IntoVec<T> for std::vec::Vec<T> {
    fn into_vec(self, env: &Env) -> Vec<T> {
        Vec::from_slice(env, self.as_slice())
    }
}
