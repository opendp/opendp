//! Verification-only experiment for the ordinary Fn / tracked-credit boundary.
//!
//! Direct tracked parameters on a closure fail in pinned Verus:
//! `with` does not support closure (probe commit c3ded5c).
//!
//! Try instead a verified named function with a ghost parameter called
//! from inside an ordinary, statically known Fn closure.
use vstd::prelude::*;

#[verus_spec(result =>
    with Ghost(expected): Ghost<u64>
    requires expected == input,
    ensures result == expected,
)]
fn named_passthrough(input: u64) -> u64 {
    input
}

#[verus_verify]
pub fn probe_named_ghost_call_through_closure(x: u64) {
    let f =
        #[verus_spec(result: u64 => ensures result == input)]
        |input: u64| {
            proof_with! { Ghost(input) }
            crate::closure_bridge_experiment::named_passthrough(input)
        };

    let out = f(x);
    proof! {
        assert(out == x);
    }
}
