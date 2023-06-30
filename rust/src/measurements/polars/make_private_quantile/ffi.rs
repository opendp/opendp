use polars::{
    prelude::{ChunkedArray, NamedFromOwned},
    series::Series,
};
use std::{
    ffi::{c_char, c_double},
    os::raw::c_void,
};

use crate::core::IntoAnyMeasurementFfiResultExt;
use crate::{
    core::{Domain, FfiResult, MetricSpace},
    domains::{AtomDomain, ExprDomain, NumericDataType, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::Type,
    },
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
    traits::{samplers::SampleUniform, DistanceConstant, Float, InfCast, Number, RoundCast},
    transformations::{traits::UnboundedMetric, DQuantileOuterMetric, ToVec},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_quantile_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    candidates: *const AnyObject,
    temperature: *const c_void,
    alpha: c_double,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<MI, TIA, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        candidates: &AnyObject,
        temperature: *const c_void,
        alpha: f64,
    ) -> Fallible<AnyMeasurement>
    where
        MI: DQuantileOuterMetric,
        MI::InnerMetric: 'static + UnboundedMetric,
        TIA: Number + NumericDataType,
        QO: InfCast<u64> + DistanceConstant<MI::Distance> + Float + SampleUniform + RoundCast<u64>,

        (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
        (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

        <ExprDomain<MI::LazyDomain> as Domain>::Carrier: Send + Sync,

        Series: NamedFromOwned<Vec<TIA>>,
        ChunkedArray<TIA::Polars>: ToVec<TIA>,
    {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<MI::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let candidates = try_!(candidates.downcast_ref::<Vec<TIA>>()).clone();
        let temperature = *try_as_ref!(temperature as *const QO);

        super::make_private_quantile_expr(
            input_domain,
            input_metric,
            candidates,
            temperature,
            alpha,
        )
        .into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let candidates = try_as_ref!(candidates);
    let alpha = alpha as f64;

    let MI = input_metric.type_.clone();
    let TIA = try_!(input_domain.get_active_column_type());
    let QO = try_!(Type::try_from(QO));

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (TIA, @numbers),
        (QO, @floats)
    ], (input_domain, input_metric, candidates, temperature, alpha)).into()
}
