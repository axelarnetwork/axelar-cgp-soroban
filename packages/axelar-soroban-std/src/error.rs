/// Return with an error if a condition is not met.
///
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

// The following macros are mostly intended to serve as pseudo-documentation within tests,
// in addition to convenience/clarity

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
            std::result::Result::Ok(v) => v,
            std::result::Result::Err(e) => {
                panic!("Error calling {}: {:?}", stringify!($x), e);
            }
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

/// Assert that a [`Result`] is [`Err`] and matches a desired error
#[macro_export]
macro_rules! assert_err {
    ( $x:expr, $e:expr ) => {
        match $x {
            std::result::Result::Err(e) => {
                if $e != e {
                    panic!("Expected error {}, got {:?} instead", stringify!($e), e)
                }
            }
            std::result::Result::Ok(v) => {
                panic!(
                    "Expected error when calling {}, got {:?} instead",
                    stringify!($x),
                    v
                );
            }
        }
    };
}
