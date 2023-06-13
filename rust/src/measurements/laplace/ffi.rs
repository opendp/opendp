use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::Type;
use crate::measurements::{make_laplace, BaseLaplaceDomain, MakeLaplace};
use crate::traits::CheckAtom;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_laplace(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T: 'static + CheckAtom, QO: 'static + Copy>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        QO: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        AtomDomain<T>: MakeLaplace<QO>,
        VectorDomain<AtomDomain<T>>: MakeLaplace<QO>,
        (
            AtomDomain<T>,
            <AtomDomain<T> as BaseLaplaceDomain>::InputMetric,
        ): MetricSpace,
        (
            VectorDomain<AtomDomain<T>>,
            <VectorDomain<AtomDomain<T>> as BaseLaplaceDomain>::InputMetric,
        ): MetricSpace,
    {
        fn monomorphize2<D: 'static + MakeLaplace<QO>, QO: 'static + Copy>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: QO,
        ) -> FfiResult<*mut AnyMeasurement>
        where
            (D, D::InputMetric): MetricSpace,
        {
            let input_domain = try_!(input_domain.downcast_ref::<D>()).clone();
            let input_metric = try_!(input_metric.downcast_ref::<D::InputMetric>()).clone();
            make_laplace::<D, QO>(input_domain, input_metric, scale).into_any()
        }
        let D = input_domain.type_.clone();
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T = try_!(input_domain.type_.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @numbers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, QO))
}

#[cfg(test)]
mod tests {
    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::ffi::util::ToCharP;
    use crate::metrics::AbsoluteDistance;

    use super::*;

    #[test]
    fn test_make_laplace_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_laplace(
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            util::into_raw(0.0) as *const c_void,
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
