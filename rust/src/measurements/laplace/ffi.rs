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
    fn monomorphize_float<T: 'static + CheckAtom + Copy>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        Q: Type,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        AtomDomain<T>: MakeLaplace<T>,
        VectorDomain<AtomDomain<T>>: MakeLaplace<T>,
        (
            AtomDomain<T>,
            <AtomDomain<T> as BaseLaplaceDomain>::InputMetric,
        ): MetricSpace,
        (
            VectorDomain<AtomDomain<T>>,
            <VectorDomain<AtomDomain<T>> as BaseLaplaceDomain>::InputMetric,
        ): MetricSpace,
    {
        fn monomorphize2<D: 'static + MakeLaplace<Q>, Q: 'static>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: Q,
        ) -> FfiResult<*mut AnyMeasurement>
        where
            (D, D::InputMetric): MetricSpace,
        {
            let input_domain = try_!(input_domain.downcast_ref::<D>()).clone();
            let input_metric = try_!(input_metric.downcast_ref::<D::InputMetric>()).clone();
            make_laplace::<D, Q>(input_domain, input_metric, scale).into_any()
        }
        let D = input_domain.type_.clone();
        let scale = *try_as_ref!(scale as *const T);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (Q, [T])
        ], (input_domain, input_metric, scale))
    }
    fn monomorphize_integer<T: 'static + CheckAtom, QO: 'static + Copy>(
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
    let QI = try_!(input_metric.distance_type.get_atom());
    let QO = try_!(Type::try_from(QO));

    // This is used to check if the type is in a dispatch set,
    // without constructing an expensive backtrace upon failed match
    fn in_set<T>() -> Option<()> {
        Some(())
    }

    if T != QI {
        return err!(
            FFI,
            "input distance type ({}) must match data type ({})",
            QI.descriptor,
            T.descriptor
        )
        .into();
    }

    if let Some(_) = dispatch!(in_set, [(T, @floats)]) {
        if T != QO {
            return err!(
                FFI,
                "since data type is float, output distance type ({}) must match data type ({})",
                QO.descriptor,
                T.descriptor
            )
            .into();
        }
        dispatch!(monomorphize_float, [
            (T, @floats)
        ], (input_domain, input_metric, scale, QO))
    } else {
        dispatch!(monomorphize_integer, [
            (T, @integers),
            (QO, @floats)
        ], (input_domain, input_metric, scale, QO))
    }
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
