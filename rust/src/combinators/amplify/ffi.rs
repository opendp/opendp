use std::os::raw::c_uint;

use opendp_derive::bootstrap;

use crate::combinators::AmplifiableMeasure;
use crate::core::FfiResult;
use crate::error::Fallible;
use crate::ffi::any::{AnyMeasure, AnyMeasurement, AnyObject, Downcast};
use crate::measures::{Approximate, MaxDivergence};

impl AmplifiableMeasure for AnyMeasure {
    fn amplify(
        &self,
        budget: &AnyObject,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<AnyObject> {
        fn monomorphize<M: 'static + AmplifiableMeasure>(
            measure: &AnyMeasure,
            budget: &AnyObject,
            population_size: usize,
            sample_size: usize,
        ) -> Fallible<AnyObject> {
            let measure = measure.downcast_ref::<M>()?;
            let budget = budget.downcast_ref::<M::Distance>()?;
            measure
                .amplify(budget, population_size, sample_size)
                .map(AnyObject::new)
        }
        dispatch!(
            monomorphize,
            [(self.type_, [MaxDivergence, Approximate<MaxDivergence>])],
            (self, budget, population_size, sample_size)
        )
    }
}

#[bootstrap(features("contrib", "honest-but-curious"))]
/// Construct an amplified measurement from a `measurement` with privacy amplification by subsampling.
/// This measurement does not perform any sampling.
/// It is useful when you have a dataset on-hand that is a simple random sample from a larger population.
///
/// The DIA, DO, MI and MO between the input measurement and amplified output measurement all match.
///
/// # Arguments
/// * `measurement` - the computation to amplify
/// * `population_size` - the size of the population from which the input dataset is a simple sample
///
/// # Why honest-but-curious?
/// The privacy guarantees are only valid if the input dataset is a simple sample from a population with `population_size` records.
fn make_population_amplification(
    measurement: &AnyMeasurement,
    population_size: usize,
) -> Fallible<AnyMeasurement> {
    super::make_population_amplification(measurement, population_size)
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_population_amplification(
    measurement: *const AnyMeasurement,
    population_size: c_uint,
) -> FfiResult<*mut AnyMeasurement> {
    make_population_amplification(try_as_ref!(measurement), population_size as usize).into()
}
