#[cfg(feature = "polars")]
use crate::{ffi::util::Type, metrics::Bounds};

use crate::{
    core::FfiResult,
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMetric, AnyObject, AnyOdometer, Downcast},
    measures::ffi::TypedMeasure,
    metrics::ffi::TypedMetric,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_combinators__make_fully_adaptive_composition(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    d_in: *const AnyObject,
) -> FfiResult<*mut AnyOdometer> {
    let input_domain = try_as_ref!(input_domain).clone();
    let input_metric = try_as_ref!(input_metric).clone();
    let output_measure = try_as_ref!(output_measure).clone();
    let d_in = try_as_ref!(d_in).clone();

    fn monomorphize<QI: 'static + Clone + Send + Sync, QO: 'static + Clone + Default>(
        input_domain: AnyDomain,
        input_metric: AnyMetric,
        output_measure: AnyMeasure,
        d_in: AnyObject,
    ) -> Fallible<AnyOdometer> {
        super::make_fully_adaptive_composition::<
            AnyDomain,
            AnyObject,
            TypedMetric<QI>,
            TypedMeasure<QO>,
        >(
            input_domain,
            TypedMetric::<QI>::new(input_metric.clone())?,
            TypedMeasure::<QO>::new(output_measure.clone())?,
            d_in.downcast::<QI>()?,
        )?
        .into_any()
    }

    let QI = input_metric.distance_type.clone();
    let QO = output_measure.distance_type.clone();

    #[cfg(feature = "polars")]
    if QI == Type::of::<Bounds>() {
        return dispatch!(
            monomorphize,
            [(QI, [Bounds]), (QO, [f64, (f64, f64)])],
            (input_domain, input_metric, output_measure, d_in)
        )
        .into();
    }
    dispatch!(monomorphize, [
        (QI, @numbers),
        (QO, [f64, (f64, f64)])
    ], (input_domain, input_metric, output_measure, d_in))
    .into()
}
