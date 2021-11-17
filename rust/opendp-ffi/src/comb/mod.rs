use std::os::raw::c_uint;

use num::Float;

use opendp::comb::{AmplifiableMeasure, IsSizedDomain, make_basic_composition, make_chain_mt, make_chain_tt, make_population_amplification};
use opendp::dist::{MaxDivergence, SmoothedMaxDivergence};
use opendp::dom::{SizedDomain, VectorDomain, AllDomain, BoundedDomain};
use opendp::err;
use opendp::error::Fallible;
use opendp::traits::{CheckNull, ExactIntCast, TotalOrd};

use crate::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyObject, AnyTransformation, Downcast, IntoAnyMeasurementOutExt};
use crate::core::FfiResult;
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_mt(measurement1: *const AnyMeasurement, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);
    make_chain_mt(measurement1, transformation0, None).into()
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_tt(transformation1: *const AnyTransformation, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyTransformation> {
    let transformation0 = try_as_ref!(transformation0);
    let transformation1 = try_as_ref!(transformation1);
    make_chain_tt(transformation1, transformation0, None).into()
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_basic_composition(measurement0: *const AnyMeasurement, measurement1: *const AnyMeasurement) -> FfiResult<*mut AnyMeasurement> {
    let measurement0 = try_as_ref!(measurement0);
    let measurement1 = try_as_ref!(measurement1);
    // This one has a different pattern than most constructors. The result of make_basic_composition()
    // will be Measurement<AnyDomain, PairDomain, AnyMetric, AnyMeasure>. We need to get back to
    // AnyMeasurement, but using IntoAnyMeasurementExt::into_any() would double-wrap the input.
    // That's what IntoAnyMeasurementOutExt::into_any_out() is for.
    make_basic_composition(measurement0, measurement1).map(IntoAnyMeasurementOutExt::into_any_out).into()
}


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
    measurement: *const AnyMeasurement, population_size: c_uint
) -> FfiResult<*mut AnyMeasurement> {
    make_population_amplification(
        try_as_ref!(measurement),
        population_size as usize).into()
}


#[cfg(test)]
mod tests {
    use opendp::core::{Function, Measurement, PrivacyRelation, Transformation};
    use opendp::dist::{MaxDivergence, SymmetricDistance};
    use opendp::dom::AllDomain;
    use opendp::error::*;
    use opendp::traits::CheckNull;
    use opendp::trans;

    use crate::any::{AnyObject, Downcast, IntoAnyMeasurementExt, IntoAnyTransformationExt};
    use crate::core;
    use crate::util;

    use super::*;

    // TODO: Find all the places we've duplicated this code and replace with common function.
    pub fn make_test_measurement<T: Clone + CheckNull>() -> Measurement<AllDomain<T>, AllDomain<T>, SymmetricDistance, MaxDivergence<f64>> {
        Measurement::new(
            AllDomain::new(),
            AllDomain::new(),
            Function::new(|arg: &T| arg.clone()),
            SymmetricDistance::default(),
            MaxDivergence::default(),
            PrivacyRelation::new(|_d_in, _d_out| true),
        )
    }

    // TODO: Find all the places we've duplicated this code and replace with common function.
    pub fn make_test_transformation<T: Clone + CheckNull>() -> Transformation<AllDomain<T>, AllDomain<T>, SymmetricDistance, SymmetricDistance> {
        trans::make_identity(AllDomain::<T>::new(), SymmetricDistance::default()).unwrap_test()
    }

    #[test]
    fn test_make_chain_mt() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>().into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any());
        let chain = Result::from(opendp_comb__make_chain_mt(measurement1, transformation0))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_make_chain_tt() -> Fallible<()> {
        let transformation0 = util::into_raw(make_test_transformation::<i32>().into_any());
        let transformation1 = util::into_raw(make_test_transformation::<i32>().into_any());
        let chain = Result::from(opendp_comb__make_chain_tt(transformation1, transformation0))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__transformation_invoke(&chain, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 999);
        Ok(())
    }

    #[test]
    fn test_make_basic_composition() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>().into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any());
        let basic_composition = Result::from(opendp_comb__make_basic_composition(measurement0, measurement1))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&basic_composition, arg);
        let res: (AnyObject, AnyObject) = Fallible::from(res)?.downcast()?;
        let res: (i32, i32) = (res.0.downcast()?, res.1.downcast()?);
        assert_eq!(res, (999, 999));
        Ok(())
    }
}
