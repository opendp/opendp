#[cfg(feature = "ffi")]
mod ffi;

use std::collections::HashMap;

use bigint::make_bigint_laplace_threshold;
use dashu::rational::RBig;

use crate::core::{Measurement, MetricSpace};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::L1Distance;
use crate::traits::{CastInternalRational, ExactIntCast, Float, Hashable, InfCast};
use crate::transformations::{
    find_next_multiple_of_2k, get_min_k, integerize_scale, make_integerize_hashmap,
    then_deintegerize_hashmap,
};

#[cfg(all(test, feature = "partials"))]
mod test;

mod bigint;

// #[bootstrap(
//     features("contrib", "floating-point"),
//     arguments(threshold(c_type = "void *"), k(default = b"null", c_type = "void *")),
//     generics(TK(suppress), TV(suppress)),
//     derived_types(TV = "$get_distance_type(input_metric)")
// )]
/// Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `scale` - Noise scale parameter for the laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `threshold` - Exclude counts that are less than this minimum value.
/// * `k` - The noise granularity in terms of 2^k.
///
/// # Generics
/// * `TK` - Type of Key. Must be hashable/categorical.
/// * `TV` - Type of Value. Must be float.
pub fn make_laplace_threshold<TK, TV>(
    input_domain: MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
    input_metric: L1Distance<TV>,
    scale: f64,
    threshold: TV,
    k: Option<i32>,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<TK>, AtomDomain<TV>>,
        HashMap<TK, TV>,
        L1Distance<TV>,
        Approximate<MaxDivergence>,
    >,
>
where
    TK: Hashable,
    TV: Float + CastInternalRational,
    i32: ExactIntCast<TV::Bits>,
    f64: InfCast<TV>,
    (MapDomain<AtomDomain<TK>, AtomDomain<TV>>, L1Distance<TV>): MetricSpace,
    RBig: TryFrom<TV>,
{
    // don't use the k set by the user for now
    let _ = k;
    let k = get_min_k::<TV>();
    let s = integerize_scale(scale, k)?;

    let threshold = find_next_multiple_of_2k(
        RBig::try_from(threshold).map_err(|_| {
            err!(
                MakeTransformation,
                "threshold ({threshold}) must be non-negative"
            )
        })?,
        k,
    );

    let t_int = make_integerize_hashmap((input_domain, input_metric), k)?;
    let t_lap = make_bigint_laplace_threshold(t_int.output_space(), s, threshold)?;

    t_int >> t_lap >> then_deintegerize_hashmap(k)
}
