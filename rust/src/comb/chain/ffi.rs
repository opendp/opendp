use crate::comb::{make_chain_mt, make_chain_tt, make_sequential_composition_static_distances};
use crate::core::FfiResult;

use crate::ffi::any::{AnyMeasurement, AnyObject, AnyTransformation, IntoAnyMeasurementOutExt};
use crate::ffi::util::AnyMeasurementPtr;
use crate::ffi::any::Downcast;

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_mt(measurement1: *const AnyMeasurement, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);
    make_chain_mt(measurement1, transformation0).into()
}

#[no_mangle]
pub extern "C" fn opendp_comb__make_chain_tt(transformation1: *const AnyTransformation, transformation0: *const AnyTransformation) -> FfiResult<*mut AnyTransformation> {
    let transformation0 = try_as_ref!(transformation0);
    let transformation1 = try_as_ref!(transformation1);
    make_chain_tt(transformation1, transformation0).into()
}


#[no_mangle]
pub extern "C" fn opendp_comb__make_sequential_composition_static_distances(
    measurements: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let meas_ptrs = try_!(try_as_ref!(measurements)
        .downcast_ref::<Vec<AnyMeasurementPtr>>());
    
    let measurements: Vec<&AnyMeasurement> = try_!(meas_ptrs.iter().map(|ptr| Ok(try_as_ref!(*ptr))).collect());

    make_sequential_composition_static_distances(measurements)
        .map(IntoAnyMeasurementOutExt::into_any_out).into()
}

#[cfg(test)]
mod tests {
    use crate::core::{Function, Measurement, PrivacyMap, Transformation};
    use crate::core;
    use crate::core::{MaxDivergence, SymmetricDistance};
    use crate::core::AllDomain;
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
            PrivacyMap::new(|_d_in| f64::INFINITY),
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
    fn test_make_sequential_composition_static_distances() -> Fallible<()> {
        let measurement0 = util::into_raw(make_test_measurement::<i32>().into_any());
        let measurement1 = util::into_raw(make_test_measurement::<i32>().into_any());
        let measurements = vec![measurement0, measurement1];
        let basic_composition = Result::from(opendp_comb__make_sequential_composition_static_distances(AnyObject::new_raw(measurements)))?;
        let arg = AnyObject::new_raw(999);
        let res = core::opendp_core__measurement_invoke(&basic_composition, arg);
        let res: (AnyObject, AnyObject) = Fallible::from(res)?.downcast()?;
        let res: (i32, i32) = (res.0.downcast()?, res.1.downcast()?);
        assert_eq!(res, (999, 999));
        Ok(())
    }
}
