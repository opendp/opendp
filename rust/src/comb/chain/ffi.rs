use std::os::raw::c_char;
use crate::comb::{make_basic_composition, make_chain_mt, make_chain_tt, make_sequential_composition_static_distances};
use crate::core::FfiResult;

use crate::ffi::any::{AnyMeasurement, AnyObject, AnyTransformation, IntoAnyMeasurementOutExt};
use crate::ffi::any::Downcast;

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
pub extern "C" fn opendp_comb__make_sequential_composition_static_distances(
    d_in: *const AnyObject,
    measurement_pairs: *const AnyObject,
    _QO: *const c_char
) -> FfiResult<*mut AnyMeasurement> {
    let d_in: &AnyObject = try_as_ref!(d_in);
    let measurement_pairs: Vec<(&AnyMeasurement, AnyObject)> = try_!(try_!(try_as_ref!(measurement_pairs)
        .downcast_ref::<Vec<AnyObject>>())
        .into_iter().map(|pair| pair.downcast_ref::<(AnyObject, AnyObject)>()
            .and_then(|(meas, dist)| Ok((meas.downcast_ref::<AnyMeasurement>()?, dist.clone()))))
        .collect());

    make_sequential_composition_static_distances(d_in.clone(), measurement_pairs)
        .map(IntoAnyMeasurementOutExt::into_any_out).into()
}

#[cfg(test)]
mod tests {
    use crate::core::{Function, Measurement, PrivacyRelation, Transformation};
    use crate::core;
    use crate::dist::{MaxDivergence, SymmetricDistance};
    use crate::dom::AllDomain;
    use crate::error::*;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast, IntoAnyMeasurementExt, IntoAnyTransformationExt};
    use crate::ffi::util;
    use crate::traits::CheckNull;
    use crate::trans;

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
