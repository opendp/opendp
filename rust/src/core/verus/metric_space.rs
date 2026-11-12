//! A reusable semantic specification for a domain-metric pairing.
//!
//! SymmetricDistance is a metric on multisets but a pseudometric on ordered
//! carriers. Equivalence at distance zero is therefore explicit.

use vstd::prelude::*;

verus! {

/// Domain and metric semantics together, independent of any transformation.
///
/// The relation equivalent identifies observationally indistinguishable
/// carriers. On tables, this means equal schemas and row multisets.
pub open spec fn valid_metric_space<X, D>(
    member: spec_fn(X) -> bool,
    equivalent: spec_fn(X, X) -> bool,
    exact_distance: spec_fn(X, X) -> nat,
    close: spec_fn(X, X, D) -> bool,
    within_bound: spec_fn(nat, D) -> bool,
) -> bool {
    &&& forall|x: X|
        #[trigger] member(x) ==> exact_distance(x, x) == 0
    &&& forall|x: X, y: X|
        #[trigger] member(x) && #[trigger] member(y) ==> {
            &&& exact_distance(x, y) == exact_distance(y, x)
            &&& (exact_distance(x, y) == 0 <==> equivalent(x, y))
        }
    &&& forall|x: X, y: X, z: X|
        #[trigger] member(x) && #[trigger] member(y) && #[trigger] member(z)
            ==> exact_distance(x, z)
                <= exact_distance(x, y) + exact_distance(y, z)
    &&& forall|x: X, y: X, d: D|
        member(x) && member(y)
            ==> #[trigger] close(x, y, d) == within_bound(exact_distance(x, y), d)
}

} // verus!
