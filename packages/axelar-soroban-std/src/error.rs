/// Return with an error if a condition is not met.
///
/// Simplifies the pattern of checking for a condition and returning with an error.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $e:expr $(,)?) => {
        if !$cond {
            return Err($e);
        }
    };
}

// The following definitions are mostly intended to serve as pseudo-documentation within tests
// and help with convenience/clarity.

/// Assert that a [`Result`] is [`Ok`]
///
/// If the provided expresion evaulates to [`Ok`], then the
/// macro returns the value contained within the [`Ok`]. If
/// the [`Result`] is an [`Err`] then the macro will [`panic`]
/// with a message that includes the expression and the error.
///
/// This function was vendored from: https://docs.rs/assert_ok/1.0.2/assert_ok/
#[macro_export]
macro_rules! assert_ok {
    ( $x:expr ) => {
        match $x {
            Ok(v) => v,
            Err(e) => {
                panic!("Error calling {}: {:?}", stringify!($x), e);
            }
        }
    };
}

/// Assert that a [`Result`] is [`Err`] and matches an error variant
#[macro_export]
macro_rules! assert_err {
    ( $x:expr, $expected:pat ) => {
        match $x {
            Err(e) => {
                if !matches!(e, $expected) {
                    panic!("Expected error {}: {:?}", stringify!($expected), e);
                }
            }
            Ok(v) => {
                panic!("Expected error {}, found {:?}", stringify!($expected), v)
            }
        }
    };
}

/// Assert that a [`Result`] from a contract call is [`Err`] and matches an error variant
///
/// `given` corresponds to the return type from `try_*` functions in Soroban.
/// For the assert to succeed, the function needs to fail and successfully pass
/// down the intended error type. So, the parameters would be in the form:
///
/// given: `Err(Ok(ContractError))`
/// expected: `ContractError`
///
/// Putting it together in a function call:
///
/// `assert_contract_err(client.try_fun(...), ContractError);`
#[macro_export]
macro_rules! assert_contract_err {
    ($given:expr, $expected:pat) => {
        match $given {
            Ok(v) => panic!(
                "Expected error {:?}, got {:?} instead",
                stringify!($expected),
                v
            ),
            Err(e) => match e {
                Ok(v) => {
                    if !matches!(v, $expected) {
                        panic!(
                            "Expected error {}, got {:?} instead",
                            stringify!($expected),
                            v
                        )
                    }
                }
                Err(e) => panic!("Unexpected error {e:?}"),
            },
        }
    };
}

/// Assert that an [`Option`] is [`Some`]
///
/// If the provided expresion evaulates to [`Some`], then the
/// macro returns the value contained within the [`Some`]. If
/// the [`Option`] is [`None`] then the macro will [`panic`]
/// with a message that includes the expression
#[macro_export]
macro_rules! assert_some {
    ( $x:expr ) => {
        match $x {
            core::option::Option::Some(s) => s,
            core::option::Option::None => {
                panic!("Expected value when calling {}, got None", stringify!($x));
            }
        }
    };
}
