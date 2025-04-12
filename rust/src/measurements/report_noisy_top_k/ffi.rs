use std::ffi::c_char;

use dashu::float::FBig;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, Downcast},
        util::to_str,
    },
    measurements::{Optimize, make_report_noisy_top_k, report_noisy_top_k::SelectionMeasure},
    measures::{MaxDivergence, RangeDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, CheckNull, DistanceConstant, Number},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_report_noisy_top_k(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    k: u32,
    scale: f64,
    optimize: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let output_measure = try_as_ref!(output_measure);
    let TIA_ = try_!(input_domain.type_.get_atom());
    let MO = output_measure.type_.clone();
    let k = k as usize;

    let optimize = try_!(Optimize::try_from(try_!(to_str(optimize))));

    fn monomorphize<MO, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        output_measure: &AnyMeasure,
        k: usize,
        scale: f64,
        optimize: Optimize,
    ) -> Fallible<AnyMeasurement>
    where
        MO: 'static + SelectionMeasure,
        TIA: Clone + CheckNull + Number + CastInternalRational,
        f64: DistanceConstant<TIA>,
        FBig: TryFrom<TIA>,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<LInfDistance<TIA>>()?.clone();
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        make_report_noisy_top_k::<MO, TIA>(
            input_domain,
            input_metric,
            output_measure,
            k,
            scale,
            optimize,
        )
        .into_any()
    }

    dispatch!(
        monomorphize,
        [
            (MO, [MaxDivergence, RangeDivergence]),
            (TIA_, [u32, u64, i32, i64, usize, f32, f64])
        ],
        (
            input_domain,
            input_metric,
            output_measure,
            k,
            scale,
            optimize
        )
    )
    .into()
}
