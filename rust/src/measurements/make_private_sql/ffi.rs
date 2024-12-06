use std::ffi::c_char;

use polars_plan::plans::DslPlan;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure},
    domains::LazyFrameDomain,
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::{self, to_str},
    },
    measurements::PrivateDslPlan,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::SymmetricDistance,
};

use super::make_private_sql;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_sql(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    query: *const c_char,
    global_scale: *const AnyObject,
    threshold: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<SymmetricDistance>()).clone();
    let output_measure = try_as_ref!(output_measure);
    let MO_ = output_measure.type_.clone();

    let query = try_!(to_str(query));

    let global_scale = if let Some(param) = util::as_ref(global_scale) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<f64>()))
    } else {
        None
    };

    let threshold = if let Some(param) = util::as_ref(threshold) {
        Some(*try_!(try_as_ref!(param).downcast_ref::<u32>()))
    } else {
        None
    };

    fn monomorphize<MO: 'static + Measure>(
        input_domain: LazyFrameDomain,
        input_metric: SymmetricDistance,
        output_measure: &AnyMeasure,
        query: &str,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<AnyMeasurement>
    where
        DslPlan: PrivateDslPlan<SymmetricDistance, MO>,
    {
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        Ok(make_private_sql(
            input_domain,
            input_metric,
            output_measure,
            query,
            global_scale,
            threshold,
        )?
        .into_any_Q()
        .into_any_A())
        .into_any()
    }

    dispatch!(
        monomorphize,
        [(MO_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>])],
        (
            input_domain,
            input_metric,
            output_measure,
            query,
            global_scale,
            threshold
        )
    )
    .into()
}
