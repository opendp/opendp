//! Validity of a measurement over its *output probability law*.
//! The output measure is a relation on laws, rather than an arbitrary
//! predicate about inputs. Fallible privacy maps impose no constraints when Err.
use vstd::prelude::*;

verus! {

pub open spec fn valid_measurement<X, Y, DI, DO, ME>(
    input_member: spec_fn(X) -> bool,
    input_close: spec_fn(X, X, DI) -> bool,
    output_law: spec_fn(X, Y) -> real,
    output_measure: spec_fn(spec_fn(Y) -> real, spec_fn(Y) -> real, DO) -> bool,
    privacy_map: spec_fn(DI) -> Result<DO, ME>,
) -> bool {
    forall|left: X, right: X, distance: DI|
        input_member(left) && input_member(right)
            && #[trigger] input_close(left, right, distance)
        ==> match privacy_map(distance) {
            Ok(cost) => output_measure(
                |outcome: Y| output_law(left, outcome),
                |outcome: Y| output_law(right, outcome),
                cost,
            ),
            Err(_) => true,
        }
}

} // verus!
