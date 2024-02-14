use std::convert::TryFrom;
use std::os::raw::{c_char, c_long, c_void};

use dashu::rational::RBig;

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::measurements::{make_base_gaussian, BaseGaussianDomain, GaussianMeasure};
use crate::measures::ZeroConcentratedDivergence;
use crate::traits::samplers::{CastInternalRational, SampleDiscreteGaussianZ2k};
use crate::traits::{ExactIntCast, Float, FloatBits};
use crate::{err, try_, try_as_ref};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_gaussian(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    k: c_long,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize1<T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        k: i32,
        D: Type,
        MO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        T: Float + CastInternalRational + SampleDiscreteGaussianZ2k,
        i32: ExactIntCast<T::Bits>,
        RBig: TryFrom<T>,
    {
        let scale = *try_as_ref!(scale as *const T);
        fn monomorphize2<D, MO>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: D::Atom,
            k: i32,
        ) -> Fallible<AnyMeasurement>
        where
            D: 'static + BaseGaussianDomain,
            (D, D::InputMetric): MetricSpace,
            D::Atom: Float + SampleDiscreteGaussianZ2k,
            MO: 'static + GaussianMeasure<D>,
            i32: ExactIntCast<<D::Atom as FloatBits>::Bits>,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_base_gaussian::<D, MO>(input_domain, input_metric, scale, Some(k)).into_any()
        }

        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (MO, [ZeroConcentratedDivergence<T>])
        ], (input_domain, input_metric, scale, k))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let k = k as i32;
    let D = input_domain.type_.clone();
    let MO = try_!(Type::try_from(MO));
    let T = try_!(D.get_atom());
    dispatch!(monomorphize1, [
        (T, @floats)
    ], (input_domain, input_metric, scale, k, D, MO))
    .into()
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;
    use crate::metrics::{AbsoluteDistance, L2Distance};

    use super::*;

    #[test]
    fn test_make_base_gaussian_vec() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_gaussian(
            AnyDomain::new_raw(VectorDomain::new(AtomDomain::<f64>::default())),
            AnyMetric::new_raw(L2Distance::<f64>::default()),
            util::into_raw(0.0) as *const c_void,
            -1078,
            "ZeroConcentratedDivergence<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![1.0, 2.0, 3.0]);
        Ok(())
    }

    #[test]
    fn test_make_base_gaussian_zcdp() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_base_gaussian(
            AnyDomain::new_raw(AtomDomain::<f64>::default()),
            AnyMetric::new_raw(AbsoluteDistance::<f64>::default()),
            util::into_raw(0.0) as *const c_void,
            -1078,
            "ZeroConcentratedDivergence<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(1.0);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 1.0);
        Ok(())
    }
}
