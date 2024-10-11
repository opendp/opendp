use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast};
use crate::ffi::util::as_ref;
use crate::measurements::{make_laplace, LaplaceDomain};
use crate::traits::CheckAtom;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_laplace(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    k: *const i32,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize_float<T: 'static + CheckAtom + Copy>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<AnyMeasurement>
    where
        AtomDomain<T>: LaplaceDomain,
        VectorDomain<AtomDomain<T>>: LaplaceDomain,
        (AtomDomain<T>, <AtomDomain<T> as LaplaceDomain>::InputMetric): MetricSpace,
        (
            VectorDomain<AtomDomain<T>>,
            <VectorDomain<AtomDomain<T>> as LaplaceDomain>::InputMetric,
        ): MetricSpace,
    {
        fn monomorphize2<D: 'static + LaplaceDomain>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: f64,
            k: Option<i32>,
        ) -> Fallible<AnyMeasurement>
        where
            (D, D::InputMetric): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_laplace::<D>(input_domain, input_metric, scale, k).into_any()
        }
        let D = input_domain.type_.clone();
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>])
        ], (input_domain, input_metric, scale, k))
    }
    fn monomorphize_integer<T: 'static + CheckAtom>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        k: Option<i32>,
    ) -> Fallible<AnyMeasurement>
    where
        AtomDomain<T>: LaplaceDomain,
        VectorDomain<AtomDomain<T>>: LaplaceDomain,
        (AtomDomain<T>, <AtomDomain<T> as LaplaceDomain>::InputMetric): MetricSpace,
        (
            VectorDomain<AtomDomain<T>>,
            <VectorDomain<AtomDomain<T>> as LaplaceDomain>::InputMetric,
        ): MetricSpace,
    {
        fn monomorphize2<D: 'static + LaplaceDomain>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: f64,
            k: Option<i32>,
        ) -> Fallible<AnyMeasurement>
        where
            (D, D::InputMetric): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_laplace::<D>(input_domain, input_metric, scale, k).into_any()
        }
        let D = input_domain.type_.clone();
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>])
        ], (input_domain, input_metric, scale, k))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let k = as_ref(k).map(Clone::clone);
    let T_ = try_!(input_domain.type_.get_atom());
    let QI = try_!(input_metric.distance_type.get_atom());

    // This is used to check if the type is in a dispatch set,
    // without constructing an expensive backtrace upon failed match
    fn in_set<T>() -> Option<()> {
        Some(())
    }

    if T_ != QI {
        return err!(
            FFI,
            "input distance type ({}) must match data type ({})",
            QI.descriptor,
            T_.descriptor
        )
        .into();
    }

    if let Some(_) = dispatch!(in_set, [(T_, @floats)]) {
        dispatch!(monomorphize_float, [
            (T_, @floats)
        ], (input_domain, input_metric, scale, k))
    } else {
        dispatch!(monomorphize_integer, [
            (T_, @integers)
        ], (input_domain, input_metric, scale, k))
    }
    .into()
}

#[cfg(test)]
mod tests {
    use std::ptr::null;

    use crate::core;
    use crate::error::Fallible;
    use crate::ffi::any::{AnyObject, Downcast};
    use crate::ffi::util;
    use crate::metrics::AbsoluteDistance;

    use super::*;

    #[test]
    fn test_make_laplace_ffi() -> Fallible<()> {
        let measurement = Result::from(opendp_measurements__make_laplace(
            util::into_raw(AnyDomain::new(AtomDomain::<i32>::default())),
            util::into_raw(AnyMetric::new(AbsoluteDistance::<i32>::default())),
            0.0,
            null(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
