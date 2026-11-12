use opendp_derive::bootstrap;

use crate::combinators::AmplifyWithoutReplacement;
use crate::core::{FfiResult, Function, PrivacyMap};
use crate::error::Fallible;
use crate::ffi::any::{
    AnyFunction, AnyMeasure, AnyMetric, AnyObject, CallbackFn, Downcast, wrap_func,
};
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::ChangeOneDistance;

#[bootstrap(
    features("contrib", "honest-but-curious"),
    arguments(
        input_metric(hint = "Metric"),
        output_measure(hint = "Measure"),
        privacy_map(rust_type = "$measure_distance_type(output_measure)"),
    ),
    returns(c_type = "FfiResult<AnyFunction *>")
)]
/// Compute the privacy map of a complete sample-level analysis after uniform
/// sampling without replacement. This is an accounting utility, not a combinator.
///
/// Both the input (sample) metric and the population metric are ChangeOneDistance.
/// The returned function applies a conservative group-privacy bound
/// at all larger population distances.
///
/// # Arguments
/// * `input_metric` - ChangeOneDistance on the fixed-size sample.
/// * `output_measure` - MaxDivergence or Approximate<MaxDivergence>.
/// * `privacy_map` - Privacy map for the complete analysis of the sample.
/// * `population_size` - Number of records in the fixed-size population.
/// * `sample_size` - Number of records sampled without replacement.
///
/// # Why honest-but-curious?
/// The privacy guarantees are only valid if the input dataset is a simple
/// random sample without replacement from a population with `population_size`
/// records, drawn independently of the data. Sample membership must not be
/// disclosed, and the supplied privacy map must account for all releases
/// involving the sample.
fn amplify_without_replacement(
    input_metric: &AnyMetric,
    output_measure: &AnyMeasure,
    privacy_map: &CallbackFn,
    population_size: usize,
    sample_size: usize,
) -> Fallible<AnyFunction> {
    input_metric.downcast_ref::<ChangeOneDistance>()?;

    fn monomorphize<MO: 'static + AmplifyWithoutReplacement>(
        output_measure: &AnyMeasure,
        privacy_map: &CallbackFn,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<AnyFunction>
    where
        MO::Distance: 'static,
    {
        let callback = wrap_func(privacy_map.clone());
        let map = PrivacyMap::new_fallible(move |d_in: &u32| {
            callback(&AnyObject::new(*d_in))?.downcast::<MO::Distance>()
        });
        let amplified = super::amplify_without_replacement(
            ChangeOneDistance,
            output_measure.downcast_ref::<MO>()?.clone(),
            map,
            population_size,
            sample_size,
        )?;
        Ok(Function::new_fallible(move |d_in: &AnyObject| {
            let d_in = if let Ok(d_in) = d_in.downcast_ref::<u32>() {
                *d_in
            } else {
                let d_in = *d_in.downcast_ref::<i32>()?;
                u32::try_from(d_in)
                    .map_err(|_| err!(InvalidDistance, "population distance must be nonnegative"))?
            };
            amplified.eval(&d_in).map(AnyObject::new)
        }))
    }

    dispatch!(
        monomorphize,
        [(output_measure.type_, [MaxDivergence, Approximate<MaxDivergence>])],
        (output_measure, privacy_map, population_size, sample_size)
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__amplify_without_replacement(
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    privacy_map: *const CallbackFn,
    population_size: usize,
    sample_size: usize,
) -> FfiResult<*mut AnyFunction> {
    amplify_without_replacement(
        try_as_ref!(input_metric),
        try_as_ref!(output_measure),
        try_as_ref!(privacy_map),
        population_size,
        sample_size,
    )
    .into()
}
