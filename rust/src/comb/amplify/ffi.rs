use std::os::raw::c_uint;

use num::Float;

use crate::comb::{AmplifiableMeasure, IsSizedDomain, make_population_amplification};
use crate::core::FfiResult;
use crate::dist::{MaxDivergence, SmoothedMaxDivergence};
use crate::dom::{AllDomain, BoundedDomain, SizedDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyObject, Downcast};
use crate::ffi::util::Type;
use crate::traits::{CheckNull, ExactIntCast, TotalOrd};

impl AmplifiableMeasure for AnyMeasure {
    fn amplify(
        &self, budget: &AnyObject, population_size: usize, sample_size: usize,
    ) -> Fallible<AnyObject> {
        fn monomorphize1<QO: 'static + Float + ExactIntCast<usize>>(
            measure: &AnyMeasure, budget: &AnyObject, population_size: usize, sample_size: usize,
        ) -> Fallible<AnyObject> {
            fn monomorphize2<M: 'static + AmplifiableMeasure>(
                measure: &AnyMeasure, budget: &AnyObject, population_size: usize, sample_size: usize,
            ) -> Fallible<AnyObject> {
                let measure = measure.downcast_ref::<M>()?;
                let budget = budget.downcast_ref::<M::Distance>()?;
                measure.amplify(budget, population_size, sample_size).map(AnyObject::new)
            }
            let measure_type = Type::of_id(&measure.measure.value.type_id())?;
            dispatch!(monomorphize2, [
                (measure_type, [MaxDivergence<QO>, SmoothedMaxDivergence<QO>])
            ], (measure, budget, population_size, sample_size))
        }

        dispatch!(monomorphize1, [(self.distance_type, @floats)], (self, budget, population_size, sample_size))
    }
}

impl IsSizedDomain for AnyDomain {
    fn get_size(&self) -> Fallible<usize> {
        fn monomorphize1<TIA>(domain: &AnyDomain, DIA: Type) -> Fallible<usize>
            where TIA: 'static + Clone + TotalOrd + CheckNull {
            fn monomorphize2<DIA: IsSizedDomain>(domain: &AnyDomain) -> Fallible<usize>
                where DIA: 'static,
                      DIA::Carrier: 'static + Clone {
                domain.downcast_ref::<DIA>()
                    .map_err(|_| err!(FFI, "failed to downcast AnyDomain to {}", Type::of::<DIA>().to_string()))?
                    .get_size()
            }

            dispatch!(monomorphize2, [(DIA, [
                SizedDomain<VectorDomain<BoundedDomain<TIA>>>,
                SizedDomain<VectorDomain<AllDomain<TIA>>>
            ])], (domain))
        }

        let DI = Type::of_id(&self.domain.value.type_id())?;
        let TIA = DI.get_atom()?;

        dispatch!(monomorphize1, [(TIA, @numbers)], (self, DI))
    }
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_population_amplification(
    measurement: *const AnyMeasurement, population_size: c_uint,
) -> FfiResult<*mut AnyMeasurement> {
    make_population_amplification(
        try_as_ref!(measurement),
        population_size as usize).into()
}

