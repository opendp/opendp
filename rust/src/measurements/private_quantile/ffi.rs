use std::ffi::c_double;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
    measurements::{TopKMeasure, make_private_quantile},
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    traits::Number,
    transformations::traits::UnboundedMetric,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_private_quantile(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    candidates: *const AnyObject,
    alpha: c_double,
    scale: c_double,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let output_measure = try_as_ref!(output_measure);
    let candidates = try_as_ref!(candidates);

    fn monomorphize<MI, MO, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        output_measure: &AnyMeasure,
        candidates: &AnyObject,
        alpha: f64,
        scale: f64,
    ) -> Fallible<AnyMeasurement>
    where
        MI: 'static + UnboundedMetric,
        MO: 'static + TopKMeasure,
        TIA: 'static + Number,
        (VectorDomain<AtomDomain<TIA>>, MI): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        let candidates = candidates.downcast_ref::<Vec<TIA>>()?.clone();
        make_private_quantile::<MI, MO, TIA>(
            input_domain,
            input_metric,
            output_measure,
            candidates,
            alpha,
            scale,
        )
        .into_any()
    }
    let MI = input_metric.type_.clone();
    let MO = output_measure.type_.clone();
    let TIA = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (MI, @dataset_metrics),
        (MO, [MaxDivergence, ZeroConcentratedDivergence]),
        (TIA, @numbers)
    ], (input_domain, input_metric, output_measure, candidates, alpha, scale))
    .into()
}
