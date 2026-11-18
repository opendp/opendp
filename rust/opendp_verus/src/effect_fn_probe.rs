//! Verus prototype: encode a linear effect in the ordinary Fn input/output
//! signature using Tracked wrappers, instead of unsupported closure `with`.
//! This is verification-only: no production API is changed.
use vstd::prelude::*;

#[verus_verify]
pub fn probe_tracked_fn(x: u64) {
    let f =
        #[verus_spec(result: (u64, Tracked<u64>) =>
            ensures
                result.0 == input,
                result.1@ == token@,
        )]
        |input: u64, token: Tracked<u64>| (input, token);

    proof_decl! { let tracked before: u64 = 7u64; }
    let (output, Tracked(after)) = f(x, Tracked(before));
    proof! {
        assert(output == x);
        assert(after == 7u64);
        assert(call_requires(f, (x, Tracked(after))));
    }
}

#[verus_verify]
pub fn probe_generic_fn<F>(f: F, x: u64)
where F: Fn(u64, Tracked<u64>) -> (u64, Tracked<u64>)
{
    proof_decl! { let tracked before: u64 = 7u64; }
    proof! {
        assert(call_requires(f, (x, Tracked(before))));
    }
    let _ = f(x, Tracked(before));
}
