#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub trait ThenOk<T, E> {
    fn then_ok(self, ok: T, err: E) -> Result<T, E>;
}

impl<T, E> ThenOk<T, E> for bool {
    fn then_ok(self, ok: T, err: E) -> Result<T, E> {
        self.then_some(ok).ok_or(err)
    }
}

#[cfg(any(test, feature = "testutils"))]
mod testutils {
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
}
