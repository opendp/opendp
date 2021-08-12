use std::convert::TryFrom;
use std::ops::{Div, Sub};
use std::os::raw::{c_char, c_uint};

use opendp::err;
use opendp::comb::{AmplifiableMeasure, make_basic_composition, make_chain_mt, make_chain_tt, make_population_amplification};
use opendp::core::{Domain, Function, Measure, Measurement, PrivacyRelation};
use opendp::dist::{MaxDivergence, SmoothedMaxDivergence};
use opendp::dom::{AllDomain, IntervalDomain, SizedDomain, VectorDomain};
use opendp::traits::{ExactIntCast, CheckNull, TotalOrd, MeasureDistance};

use crate::any::{AnyMeasurement, AnyTransformation, IntoAnyMeasurementOutExt, AnyMetricDistance, AnyMeasureDistance, Downcast};
use crate::core::FfiResult;
use crate::util::Type;
use num::Float;
use std::fmt::Debug;

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

#[no_mangle]
pub extern "C" fn opendp_comb__make_population_amplification(
    measurement: *const AnyMeasurement, n_population: c_uint,
    DIA: *const c_char, MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let measurement = try_as_ref!(measurement);
    let DIA = try_!(Type::try_from(DIA));
    let TIA = try_!(DIA.get_atom());
    let MO = try_!(Type::try_from(MO));
    let QO = try_!(MO.get_atom());
    let n_population = n_population as usize;

    fn monomorphize<TIA, QO>(
        measurement: &AnyMeasurement, n_population: usize,
        DIA: Type, MO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
        where TIA: 'static + Clone + TotalOrd + CheckNull + Debug,
              QO: 'static + ExactIntCast<usize> + Div<Output=QO> + Clone + Float + ExactIntCast<usize> + MeasureDistance + for<'a> Sub<&'a QO, Output=QO> {
        fn monomorphize2<DIA: Domain, MO: Measure>(
            measurement: &AnyMeasurement, n_population: usize,
        ) -> FfiResult<*mut AnyMeasurement>
            where MO: 'static + AmplifiableMeasure,
                  MO::Atom: ExactIntCast<usize> + Div<Output=MO::Atom> + Clone,
                  MO::Distance: Clone + PartialOrd + MeasureDistance,
                  DIA: 'static,
                  DIA::Carrier: 'static + Clone {

            // reverse the conversion to Any on the input domain and output measure
            let temp_measurement = Measurement::new(
                try_!(measurement.input_domain.domain.value
                    .downcast_ref::<SizedDomain<VectorDomain<DIA>>>()
                    .ok_or_else(|| err!(FFI, "failed to downcast AnyDomain to SizedDomain<VectorDomain<{}>>", Type::of::<DIA>().to_string()))).clone(),
                measurement.output_domain.clone(),
                Function::new(|_| unreachable!()),
                measurement.input_metric.clone(),
                try_!(measurement.output_measure.measure.value
                    .downcast_ref::<MO>()
                    .ok_or_else(|| err!(FFI, "failed to downcast AnyMeasure to MO"))).clone(),
                {
                    let privacy_relation = measurement.privacy_relation.clone();
                    PrivacyRelation::new_fallible(move |d_in: &AnyMetricDistance, d_out: &MO::Distance|
                        privacy_relation.eval(d_in, &AnyMeasureDistance::new(d_out.clone())))
                });

            // use the population amplification constructor to replace the privacy relation
            let adjusted_relation = try_!(make_population_amplification(&temp_measurement, n_population)).privacy_relation;
            let mut measurement = measurement.clone();
            measurement.privacy_relation = PrivacyRelation::new_fallible(
                move |d_in: &AnyMetricDistance, d_out: &AnyMeasureDistance|
                    adjusted_relation.eval(d_in, d_out.downcast_ref()?));
            Ok(measurement).into()
        }
        dispatch!(monomorphize2, [
            (DIA, [IntervalDomain<TIA>, AllDomain<TIA>]),
            (MO, [MaxDivergence<QO>, SmoothedMaxDivergence<QO>])
        ], (measurement, n_population))
    }
    // if DIA is AllDomain, then TIA should dispatch to @primitives, otherwise @numbers
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (QO, @floats)
    ], (measurement, n_population, DIA, MO))
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
