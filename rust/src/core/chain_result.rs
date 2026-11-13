//! Common executable short-circuiting for Function, StabilityMap and
//! PrivacyMap composition. This same source is checked by Verus.
//!
//! The verification-only attribute does not select a different implementation:
//! ordinary Rust and Verus execute the same match.
#[cfg(verus_keep_ghost)]
use vstd::prelude::*;

#[cfg_attr(verus_keep_ghost, verus_spec(output =>
    requires
        match intermediate {
            Ok(value) => call_requires(outer, (&value,)),
            Err(_) => true,
        },
    ensures
        match intermediate {
            Err(error) => output == Err(error),
            Ok(value) => call_ensures(outer, (&value,), output),
        },
))]
pub(crate) fn chain_result<T, U, E>(
    intermediate: Result<T, E>,
    outer: impl Fn(&T) -> Result<U, E>,
) -> Result<U, E> {
    match intermediate {
        Ok(value) => outer(&value),
        Err(error) => Err(error),
    }
}

/// Verified higher-order composition for any callbacks whose call contracts
/// are available. This same implementation is used by native OpenDP.
///
/// In native Function::make_chain the callbacks are already Arc<dyn Fn>;
/// this theorem proves the call path conditional on those contracts, not the
/// correctness of arbitrary erased callbacks. The inner callback must be
/// callable here; the outer only for permitted successful inner results.
#[cfg_attr(verus_keep_ghost, verus_spec(output =>
    requires
        call_requires(inner, (input,)),
        forall|middle: Result<M, E>| #![trigger call_ensures(inner, (input,), middle)]
            call_ensures(inner, (input,), middle)
            ==> match middle {
                Ok(value) => call_requires(outer, (&value,)),
                Err(_) => true,
            },
    ensures
        exists|middle: Result<M, E>| #![trigger call_ensures(inner, (input,), middle)]
            call_ensures(inner, (input,), middle)
            && match middle {
                Ok(value) => call_ensures(outer, (&value,), output),
                Err(error) => output == Err(error),
            },
))]
pub(crate) fn chain_callbacks<T, M, U, E>(
    input: &T,
    inner: impl Fn(&T) -> Result<M, E>,
    outer: impl Fn(&M) -> Result<U, E>,
) -> Result<U, E> {
    chain_result(inner(input), outer)
}

#[cfg(test)]
mod tests {
    use super::chain_result;
    use std::cell::Cell;

    #[test]
    fn preserves_result_and_invokes_outer_only_on_success() {
        let calls = Cell::new(0);
        let outer = |value: &i32| {
            calls.set(calls.get() + 1);
            Ok::<_, &'static str>(value + 1)
        };
        assert_eq!(chain_result(Err::<i32, _>("inner failed"), outer),
                   Err("inner failed"));
        assert_eq!(calls.get(), 0);
        assert_eq!(chain_result(Ok(3), outer), Ok(4));
        assert_eq!(calls.get(), 1);
    }
}
