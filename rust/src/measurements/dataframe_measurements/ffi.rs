use std::convert::TryFrom;
use std::os::raw::{c_char, c_long, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::measurements::{make_base_gaussian, GaussianDomain, GaussianMeasure};
use crate::measures::ZeroConcentratedDivergence;
use crate::traits::samplers::{CastInternalRational, SampleDiscreteGaussianZ2k};
use crate::traits::{ExactIntCast, Float, FloatBits};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_polarsDF_laplace(
    input_domain: *const c_void,
    input_metric: *const c_void,
    scale: *const c_void,
    k: c_long,
    T: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize1<T>(
        input_domain: *const c_void,
        input_metric: *const c_void,
        scale: *const c_void,
        k: i32,
        T: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        //T: Float + CastInternalRational + SampleDiscreteGaussianZ2k,
        i32: ExactIntCast<f64::Bits>,
        //rug::Rational: TryFrom<T>,
    {
        let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<L1Distance::<f64>>()).clone();

        let scale = *try_as_ref!(scale as *const T);  

        let k = k as i32;
        let T = try_!(D.get_atom());

        let meas = try_!(make_polarsDF_laplace(
            input_domain,
            input_metric,
            scale,
            k
        ));

        Ok(Measurement::new(
            meas.input_domain,
            Function::new_fallible(move |arg: &LazyFrame| {
                let res = function.eval(arg);
                res.map(|o| {
                    o.into_iter()
                        .map(AnyObject::new)
                        .collect::<Vec<AnyObject>>()
                })
            }),
            meas.input_metric,
            meas.output_metric,
            StabilityMap::new_fallible(move |d_in: &IntDistance| {
                let k = stability_map.eval(d_in)?;
                Ok(AnyObject::new(k))
            }),
        ))
        .into_any()

    }

        dispatch!(monomorphize, [
            (T, @????)
        ], (input_domain, partition_column, sum_column, bounds, null_partition))
    
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_polarsDF_laplace() -> Fallible<()> {
        
        Ok(())
    }
}
