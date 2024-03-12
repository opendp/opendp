use std::os::raw::c_uint;

use opendp_derive::bootstrap;

use crate::combinators::{AmplifiableMeasure, IsSizedDomain};
use crate::core::FfiResult;
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::Type;
use crate::measures::{FixedSmoothedMaxDivergence, MaxDivergence};
use crate::traits::{CheckAtom, ExactIntCast, InfDiv, InfExpM1, InfLn1P, InfMul, ProductOrd};

impl AmplifiableMeasure for AnyMeasure {
    fn amplify(
        &self,
        budget: &AnyObject,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<AnyObject> {
        fn monomorphize1<
            QO: 'static + ExactIntCast<usize> + InfMul + InfExpM1 + InfLn1P + InfDiv + Clone,
        >(
            measure: &AnyMeasure,
            budget: &AnyObject,
            population_size: usize,
            sample_size: usize,
        ) -> Fallible<AnyObject> {
            fn monomorphize2<M: 'static + AmplifiableMeasure>(
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
            let measure_type = Type::of_id(&measure.measure.value.type_id())?;
            dispatch!(monomorphize2, [
                (measure_type, [MaxDivergence<QO>, FixedSmoothedMaxDivergence<QO>])
            ], (measure, budget, population_size, sample_size))
        }

        dispatch!(monomorphize1, [(self.distance_type, @floats)], (self, budget, population_size, sample_size))
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
/// Protected by the "honest-but-curious" feature flag
/// because a dishonest adversary could set the population size to be arbitrarily large.
///
/// # Arguments
/// * `measurement` - the computation to amplify
/// * `population_size` - the size of the population from which the input dataset is a simple sample
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
