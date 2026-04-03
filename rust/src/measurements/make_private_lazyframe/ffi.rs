use opendp_derive::bootstrap;
use polars::{lazy::frame::LazyFrame, prelude::DslPlan};

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt, Measure, Metric, MetricSpace},
    domains::{DatabaseDomain, DslPlanDomain, LazyFrameDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util,
    },
    measurements::{PrivateDslPlan, make_private_lazyframe::group_by::ApproximateMeasure},
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::{
        ChangeOneDistance, ChangeOneIdDistance, DatabaseIdDistance, FrameDistance,
        HammingDistance, InsertDeleteDistance, L01InfDistance, MicrodataMetric, SymmetricDistance,
        SymmetricIdDistance,
    },
    transformations::{StableDslPlan, StableExpr},
};

use super::{make_private_database_lazyframe, make_private_lazyframe};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_private_lazyframe(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    lazyframe: *const AnyObject,
    global_scale: *const AnyObject,
    threshold: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_!(try_as_ref!(input_domain).downcast_ref::<LazyFrameDomain>()).clone();
    let input_metric = try_as_ref!(input_metric);
    let output_measure = try_as_ref!(output_measure);
    let MI_ = input_metric.type_.clone();
    let MO_ = output_measure.type_.clone();

    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();

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

    fn monomorphize<MI: 'static + Metric, MO: 'static + Measure>(
        input_domain: LazyFrameDomain,
        input_metric: &AnyMetric,
        output_measure: &AnyMeasure,
        lazyframe: LazyFrame,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<AnyMeasurement>
    where
        DslPlan: PrivateDslPlan<MI, MO>,
        (LazyFrameDomain, MI): MetricSpace,
        (DslPlanDomain, MI): MetricSpace,
    {
        let input_metric = input_metric.downcast_ref::<MI>()?.clone();
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        Ok(make_private_lazyframe(
            input_domain,
            input_metric,
            output_measure,
            lazyframe,
            global_scale,
            threshold,
        )?
        .into_any_Q()
        .into_any_A())
        .into_any()
    }

    dispatch!(
        monomorphize,
        [
            (MI_, [
                SymmetricDistance, SymmetricIdDistance, InsertDeleteDistance,
                ChangeOneDistance, HammingDistance, ChangeOneIdDistance,
                FrameDistance<SymmetricDistance>, FrameDistance<SymmetricIdDistance>, FrameDistance<InsertDeleteDistance>
            ]),
            (MO_, [MaxDivergence, Approximate<MaxDivergence>, ZeroConcentratedDivergence, Approximate<ZeroConcentratedDivergence>])],
        (
            input_domain,
            input_metric,
            output_measure,
            lazyframe,
            global_scale,
            threshold
        )
    )
    .into()
}

#[bootstrap(
    features("contrib"),
    returns(c_type = "FfiResult<AnyMeasurement *>")
)]
#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_private_database_lazyframe(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_measure: *const AnyMeasure,
    lazyframe: *const AnyObject,
    global_scale: *const AnyObject,
    threshold: *const AnyObject,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain =
        try_!(try_as_ref!(input_domain).downcast_ref::<DatabaseDomain>()).clone();
    let input_metric = try_!(try_as_ref!(input_metric).downcast_ref::<DatabaseIdDistance>()).clone();
    let output_measure = try_as_ref!(output_measure);
    let MO_ = output_measure.type_.clone();
    let lazyframe = try_!(try_as_ref!(lazyframe).downcast_ref::<LazyFrame>()).clone();

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

    fn monomorphize<MO: 'static + Measure + crate::combinators::CompositionMeasure + ApproximateMeasure>(
        input_domain: DatabaseDomain,
        input_metric: DatabaseIdDistance,
        output_measure: &AnyMeasure,
        lazyframe: LazyFrame,
        global_scale: Option<f64>,
        threshold: Option<u32>,
    ) -> Fallible<AnyMeasurement>
    where
        polars::prelude::Expr: crate::measurements::PrivateExpr<
                L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
                MO,
            > + StableExpr<
                L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
                L01InfDistance<<SymmetricIdDistance as MicrodataMetric>::EventMetric>,
            >,
        DslPlan: StableDslPlan<FrameDistance<SymmetricIdDistance>, FrameDistance<SymmetricDistance>>,
        (DatabaseDomain, DatabaseIdDistance): MetricSpace,
    {
        let output_measure = output_measure.downcast_ref::<MO>()?.clone();
        Ok(make_private_database_lazyframe(
            input_domain,
            input_metric,
            output_measure,
            lazyframe,
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
            lazyframe,
            global_scale,
            threshold
        )
    )
    .into()
}
