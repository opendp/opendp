//! Generic validity predicate for OpenDP Measurements.
//! The output relation compares distributions induced by pairs of inputs.
use vstd::prelude::*;

verus! {

pub open spec fn valid_measurement<X, DI, DO, ME>(
    input_member: spec_fn(X) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    output_distributions_close: spec_fn(X, X, DO) -> bool,
    privacy_map: spec_fn(DI) -> Result<DO, ME>,
) -> bool {
    forall|left: X, right: X, distance: DI|
        input_member(left) && input_member(right)
            && #[trigger] input_close(left, right, distance)
        ==> match privacy_map(distance) {
            Ok(cost) => output_distributions_close(left, right, cost),
            Err(_) => true,
        }
}

} // verus!
