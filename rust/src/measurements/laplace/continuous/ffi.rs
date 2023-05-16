use std::os::raw::{c_long, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::measurements::{make_base_laplace, BaseLaplaceDomain};
use crate::traits::samplers::SampleDiscreteLaplaceZ2k;
use crate::traits::{ExactIntCast, Float, FloatBits};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_laplace(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    k: c_long,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<D>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        k: i32,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        D: 'static + BaseLaplaceDomain,
        (D, D::InputMetric): MetricSpace,
        D::Atom: Float + SampleDiscreteLaplaceZ2k,
        i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
    {
        let input_domain = try_!(input_domain.downcast_ref::<D>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<D::InputMetric>()).clone();
        let scale = *try_as_ref!(scale as *const D::Atom);
        make_base_laplace::<D>(input_domain, input_metric, scale, Some(k)).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let k = k as i32;
    let D = input_domain.type_.clone();
    dispatch!(monomorphize, [
        (D, [AtomDomain<f64>, AtomDomain<f32>, VectorDomain<AtomDomain<f64>>, VectorDomain<AtomDomain<f32>>])
    ], (input_domain, input_metric, scale, k))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::metrics::{AbsoluteDistance, L1Distance};

    use super::*;

    #[test]
    fn test_make_base_laplace() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_laplace(
            util::into_raw(AnyDomain::new(AtomDomain::<f64>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<f64>::default())),
            util::into_raw(0.0) as *const c_void,
            -1078,
        ))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_laplace(
            util::into_raw(AnyDomain::new(VectorDomain::new(
                AtomDomain::<f64>::default(),
            ))),
            util::into_raw(AnyMetric::new(L1Distance::<f64>::default())),
            util::into_raw(0.0) as *const c_void,
            -1078,
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }
}
