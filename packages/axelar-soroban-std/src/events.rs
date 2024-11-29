use core::fmt::Debug;
use soroban_sdk::{Env, IntoVal, Topics, Val};

#[cfg(any(test, feature = "testutils"))]
pub use testutils::*;

pub trait Event: Debug + PartialEq {
    type Data: IntoVal<Env, Val> + Debug;
    type Topics: Topics + Debug;

    fn topics(&self) -> Self::Topics;

    fn data(&self) -> Self::Data;

    fn emit(&self, env: &Env) {
        env.events().publish(self.topics(), self.data());
    }

    #[cfg(any(test, feature = "testutils"))]
    fn matches(
        &self,
        env: &Env,
        event: &(soroban_sdk::Address, soroban_sdk::Vec<Val>, Val),
    ) -> bool;

    #[cfg(any(test, feature = "testutils"))]
    fn standardized_fmt(
        env: &Env,
        event: &(soroban_sdk::Address, soroban_sdk::Vec<Val>, Val),
    ) -> std::string::String;
}

#[cfg(any(test, feature = "testutils"))]
mod testutils {
    use crate::events::Event;
    use soroban_sdk::testutils::Events;
    use soroban_sdk::Env;

    pub fn fmt_last_emitted_event<E>(env: &Env) -> std::string::String
    where
        E: Event,
    {
        let event = env.events().all().last().expect("no event found");
        E::standardized_fmt(env, &event)
    }

    pub fn fmt_emitted_event_at_idx<E>(env: &Env, idx: u32) -> std::string::String
    where
        E: Event,
    {
        let event = env
            .events()
            .all()
            .get(idx)
            .expect("no event found at the given index");
        E::standardized_fmt(env, &event)
    }

    #[macro_export]
    macro_rules! impl_testutils {
        (($($topic_type:ty),*), ($($data_type:ty),*)) => {
            fn matches(&self, env: &soroban_sdk::Env, event: &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> bool {
                use soroban_sdk::IntoVal;

                Self::standardized_fmt(env, event) == Self::standardized_fmt(env, &(event.0.clone(), self.topics().into_val(env), self.data().into_val(env)))
            }

            #[allow(unused_assignments)]
            fn standardized_fmt(env: &soroban_sdk::Env, (contract_id, topics, data): &(soroban_sdk::Address, soroban_sdk::Vec<soroban_sdk::Val>, soroban_sdk::Val)) -> std::string::String {
                use soroban_sdk::TryFromVal;

                let mut topics_output = std::vec![];

                let mut i = 0;
                $(
                    let topic = topics.get(i).expect("the number of topics does not match this function's definition");
                    topics_output.push(std::format!("{:?}", <$topic_type>::try_from_val(env, &topic)
                        .expect("given topic value does not match the expected type")));

                    i += 1;
                )*

                let data = soroban_sdk::Vec::<soroban_sdk::Val>::try_from_val(env, data).expect("data should be defined as a vector-compatible type");

                let mut data_output = std::vec![];


                let mut i = 0;
                $(
                    let data_entry = data.get(i).expect("the number of data entries does not match this function's definition");
                    data_output.push(std::format!("{:?}", <$data_type>::try_from_val(env, &data_entry)
                        .expect("given data value does not match the expected type")));

                    i += 1;
                )*

                std::format!("contract: {:?}, topics: ({}), data: ({})", contract_id, topics_output.join(", "), data_output.join(", "))
            }
        };
    }
}

#[cfg(test)]
mod test {
    use crate::events::Event;
    use crate::{events, impl_testutils};
    use soroban_sdk::testutils::Events;
    use soroban_sdk::xdr::Int32;
    use soroban_sdk::{contract, BytesN, String, Symbol};

    #[derive(Debug, PartialEq, Eq)]
    struct TestEvent {
        topic1: Symbol,
        topic2: String,
        topic3: Int32,
        data1: String,
        data2: BytesN<32>,
    }

    impl Event for TestEvent {
        type Data = (String, BytesN<32>);
        type Topics = (Symbol, String, Int32);

        fn topics(&self) -> Self::Topics {
            (self.topic1.clone(), self.topic2.clone(), self.topic3)
        }

        fn data(&self) -> Self::Data {
            (self.data1.clone(), self.data2.clone())
        }

        #[cfg(any(test, feature = "testutils"))]
        impl_testutils!((Symbol, String, Int32), (String, BytesN<32>));
    }

    #[contract]
    struct Contract;

    #[test]
    fn test_format_last_emitted_event() {
        let env = soroban_sdk::Env::default();
        let expected = TestEvent {
            topic1: Symbol::new(&env, "topic1"),
            topic2: String::from_str(&env, "topic2"),
            topic3: 10,
            data1: String::from_str(&env, "data1"),
            data2: BytesN::from_array(&env, &[3; 32]),
        };

        let contract = env.register(Contract, ());
        env.as_contract(&contract, || {
            expected.emit(&env);
        });

        assert!(expected.matches(&env, &env.events().all().last().unwrap()));

        goldie::assert!(events::fmt_last_emitted_event::<TestEvent>(&env));
    }
}
