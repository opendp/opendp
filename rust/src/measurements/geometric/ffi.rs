use std::ffi::{c_char, c_void};

use super::LaplaceDomain;
use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::{as_ref, Type},
    },
    measurements::{make_geometric, GeometricDomain},
    traits::CheckAtom,
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_geometric(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const c_void,
    bounds: *const AnyObject,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize_integer<T: 'static + CheckAtom, QO: 'static + Copy>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: *const c_void,
        bounds: *const AnyObject,
        QO: Type,
    ) -> Fallible<AnyMeasurement>
    where
        AtomDomain<T>: GeometricDomain<QO>,
        <AtomDomain<T> as LaplaceDomain<QO>>::InputMetric: Metric<Distance = T>,
        VectorDomain<AtomDomain<T>>: GeometricDomain<QO>,
        <VectorDomain<AtomDomain<T>> as LaplaceDomain<QO>>::InputMetric: Metric<Distance = T>,
        (
            AtomDomain<T>,
            <AtomDomain<T> as LaplaceDomain<QO>>::InputMetric,
        ): MetricSpace,
        (
            VectorDomain<AtomDomain<T>>,
            <VectorDomain<AtomDomain<T>> as LaplaceDomain<QO>>::InputMetric,
        ): MetricSpace,
    {
        fn monomorphize2<D: 'static + GeometricDomain<QO>, QO: 'static + Copy>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            scale: QO,
            bounds: Option<(
                <D::InputMetric as Metric>::Distance,
                <D::InputMetric as Metric>::Distance,
            )>,
        ) -> Fallible<AnyMeasurement>
        where
            (D, D::InputMetric): MetricSpace,
        {
            let input_domain = input_domain.downcast_ref::<D>()?.clone();
            let input_metric = input_metric.downcast_ref::<D::InputMetric>()?.clone();
            make_geometric::<D, QO>(input_domain, input_metric, scale, bounds).into_any()
        }
        let D = input_domain.type_.clone();
        let bounds = if let Some(bounds) = as_ref(bounds) {
            Some(try_!(bounds.downcast_ref::<(T, T)>()).clone())
        } else {
            None
        };
        let scale = *try_as_ref!(scale as *const QO);
        dispatch!(monomorphize2, [
            (D, [AtomDomain<T>, VectorDomain<AtomDomain<T>>]),
            (QO, [QO])
        ], (input_domain, input_metric, scale, bounds))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let T = try_!(input_domain.type_.get_atom());
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize_integer, [
        (T, @integers),
        (QO, @floats)
    ], (input_domain, input_metric, scale, bounds, QO))
    .into()
}
