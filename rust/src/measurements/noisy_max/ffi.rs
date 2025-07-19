use std::ffi::c_char;

use dashu::float::FBig;

#[allow(deprecated)]
use crate::measurements::make_report_noisy_max_gumbel;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, Downcast},
        util::{c_bool, to_bool, to_str},
    },
    measurements::{Optimize, make_noisy_max, noisy_top_k::TopKMeasure},
    measures::{MaxDivergence, ZeroConcentratedDivergence},
    metrics::LInfDistance,
    traits::{CastInternalRational, CheckNull, DistanceConstant, Number},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_noisy_max(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    scale: f64,
    negate: c_bool,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let output_measure = try_as_ref!(output_measure);
    let TIA_ = try_!(input_domain.type_.get_atom());
    let MO = output_measure.type_.clone();

    let negate = to_bool(negate);

    fn monomorphize<MO, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        output_measure: &AnyMeasure,
        scale: f64,
        negate: bool,
    ) -> Fallible<AnyMeasurement>
    where
        MO: 'static + TopKMeasure,
        TIA: Clone + CheckNull + Number + CastInternalRational,
        f64: DistanceConstant<TIA>,
        FBig: TryFrom<TIA>,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<LInfDistance<TIA>>()?.clone();
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        make_noisy_max::<MO, TIA>(input_domain, input_metric, output_measure, scale, negate)
            .into_any()
    }

    dispatch!(
        monomorphize,
        [
            (MO, [MaxDivergence, ZeroConcentratedDivergence]),
            (TIA_, [u32, u64, i32, i64, usize, f32, f64])
        ],
        (input_domain, input_metric, output_measure, scale, negate)
    )
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_report_noisy_max_gumbel(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    optimize: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA_ = try_!(input_domain.type_.get_atom());

    let optimize = try_!(Optimize::try_from(try_!(to_str(optimize))));

    fn monomorphize<TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        optimize: Optimize,
    ) -> Fallible<AnyMeasurement>
    where
        TIA: Clone + CheckNull + Number + CastInternalRational,
        f64: DistanceConstant<TIA>,
        FBig: TryFrom<TIA>,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<LInfDistance<TIA>>()?.clone();
        #[allow(deprecated)]
        make_report_noisy_max_gumbel::<TIA>(input_domain, input_metric, scale, optimize).into_any()
    }

    dispatch!(
        monomorphize,
        [(TIA_, [u32, u64, i32, i64, usize, f32, f64])],
        (input_domain, input_metric, scale, optimize)
    )
    .into()
}
