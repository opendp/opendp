use std::os::raw::c_uint;

use opendp_derive::bootstrap;

use crate::combinators::{AmplifiableMeasure, IsSizedDomain};
use crate::core::FfiResult;
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::Type;
use crate::measures::{Approximate, MaxDivergence};
use crate::traits::{CheckAtom, ProductOrd};

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

impl IsSizedDomain for AnyDomain {
    fn get_size(&self) -> Fallible<usize> {
        fn monomorphize1<TIA>(domain: &AnyDomain, DIA: Type) -> Fallible<usize>
        where
            TIA: 'static + Clone + ProductOrd + CheckAtom,
        {
            fn monomorphize2<DIA: IsSizedDomain>(domain: &AnyDomain) -> Fallible<usize>
            where
                DIA: 'static,
                DIA::Carrier: 'static + Clone,
            {
                domain
                    .downcast_ref::<DIA>()
                    .map_err(|_| {
                        err!(
                            FFI,
                            "failed to downcast AnyDomain to {}",
                            Type::of::<DIA>().to_string()
                        )
                    })?
                    .get_size()
            }

            dispatch!(
                monomorphize2,
                [(DIA, [VectorDomain<AtomDomain<TIA>>])],
                (domain)
            )
        }

        let DI = Type::of_id(&self.domain.value.type_id())?;
        let TIA = DI.get_atom()?;

        dispatch!(monomorphize1, [(TIA, @numbers)], (self, DI))
    }
}

#[bootstrap(
    features("contrib", "honest-but-curious"),
    dependencies("$get_dependencies(measurement)")
)]
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

#[no_mangle]
pub extern "C" fn opendp_combinators__make_population_amplification(
    measurement: *const AnyMeasurement,
    population_size: c_uint,
) -> FfiResult<*mut AnyMeasurement> {
    make_population_amplification(try_as_ref!(measurement), population_size as usize).into()
}
