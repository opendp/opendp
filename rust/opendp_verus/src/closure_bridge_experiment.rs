//! Verification-only experiment: can an ordinary executable Fn closure
//! carry a tracked proof resource through its invocation?
//!
//! This deliberately tests the frontend boundary before using ErrorCredit
//! or making any probability-law claim.
use vstd::prelude::*;

#[verus_verify]
pub fn probe_closure_tracked_forwarding(x: u64) {
    let f =
        #[verus_spec(result: u64 =>
            with Tracked(start): Tracked<u64> -> end: Tracked<u64>
            ensures
                result == input,
                end == start,
        )]
        |input: u64| input;

    proof_decl! {
        let tracked credit: u64 = 7;
    }
    proof_with! { Tracked(credit) => Tracked(returned_credit) }
    let out = f(x);
    proof! {
        assert(out == x);
        assert(returned_credit == 7);
    }
}
