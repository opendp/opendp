//! Verus-only probe: put linear effect resources in an ordinary Fn's
//! argument and result types, avoiding unsupported `with` on closures.
//! No native/OpenDP measurement contract is changed by this experiment.
use vstd::prelude::*;

verus! {

pub fn probe_tracked_fn(x: u64) {
    let f = |input: u64, token: Tracked<u64>| -> (result: (u64, Tracked<u64>))
        ensures
            result.0 == input,
            result.1@ == token@,
    {
        (input, token)
    };

    let tracked before: u64 = 7;
    let (output, Tracked(after)) = f(x, Tracked(before));
    assert(output == x);
    assert(after == 7);
}

pub fn probe_generic_fn<F>(
    f: F,
    x: u64,
    token: Tracked<u64>,
) -> (result: (u64, Tracked<u64>))
    where F: Fn(u64, Tracked<u64>) -> (u64, Tracked<u64>),
    requires
        f.requires((x, token)),
        forall|out: (u64, Tracked<u64>)|
            f.ensures((x, token), out) ==> out.0 == x,
    ensures result.0 == x,
{
    f(x, token)
}

} // verus!
