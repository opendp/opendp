//! Generic pairwise validity for pure-DP Measurements with discrete laws.
//! Each input induces a law on observable outcomes (including errors).
//! For finite epsilon, MaxDivergence reduces to pointwise domination;
//! the separate Option::None outcome represents infinite epsilon.
//! The output-law semantics must be justified by a sampler-correctness proof.
use vstd::prelude::*;

verus! {

pub open spec fn valid_max_divergence_measurement<X, Y, DI, ME>(
    input_member: spec_fn(X) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    output_law: spec_fn(X, Y) -> real,
    privacy_map: spec_fn(DI) -> Result<Option<real>, ME>,
) -> bool {
    forall|left: X, right: X, distance: DI|
        input_member(left) && input_member(right)
            && #[trigger] input_close(left, right, distance)
        ==> match privacy_map(distance) {
            Ok(Some(factor)) => forall|outcome: Y|
                #[trigger] output_law(left, outcome) <= factor * output_law(right, outcome),
            Ok(None) | Err(_) => true,
        }
}

} // verus!
