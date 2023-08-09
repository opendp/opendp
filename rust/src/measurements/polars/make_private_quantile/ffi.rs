use std::{os::raw::c_void, ffi::c_char};
use polars::{prelude::{ChunkedArray, NamedFromOwned}, series::Series};

use crate::{
    core::{FfiResult, MetricSpace, Domain},
    domains::{ExprDomain, NumericDataType, VectorDomain, AtomDomain},
    ffi::{
        any::{AnyDomain, AnyMetric, AnyMeasurement, Downcast, AnyObject}, 
        util::Type
    }, 
    transformations::{DQuantileOuterMetric, ARDatasetMetric, IntoFrac, ToVec}, 
    metrics::{L1, InsertDeleteDistance, SymmetricDistance},
    traits::{Float, InfCast, Number, DistanceConstant}
};
use crate::core::IntoAnyMeasurementFfiResultExt;

#[no_mangle]
pub extern "C" fn opendp_measurements__make_private_quantile(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    candidates: *const AnyObject,
    temperature: *const c_void,
    alpha: *const AnyObject,
    QO: *const c_char,
    A: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {

    fn monomorphize<MI, TIA, QO, A>(
        input_domain: &AnyDomain, 
        input_metric: &AnyMetric, 
        candidates: &AnyObject,
        temperature: *const c_void,
        alpha: &AnyObject,
    ) -> FfiResult<*mut AnyMeasurement>
    where 
        MI: DQuantileOuterMetric,
        MI::InnerMetric: 'static + ARDatasetMetric,
        TIA: Number + NumericDataType,
        QO: InfCast<u64> + DistanceConstant<MI::Distance> + Float,
        A: Clone + IntoFrac,

        (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
        (ExprDomain<MI::LazyDomain>, MI::ScoreMetric): MetricSpace,
        (VectorDomain<AtomDomain<TIA>>, MI::InnerMetric): MetricSpace,

        <ExprDomain<MI::LazyDomain> as Domain>::Carrier: Send + Sync,

        Series: NamedFromOwned<Vec<TIA>>,
        ChunkedArray<TIA::Polars>: ToVec<TIA>,
    {
        let input_domain =
            try_!(input_domain.downcast_ref::<ExprDomain<MI::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        let candidates = try_!(candidates.downcast_ref::<Vec<TIA>>()).clone();
        let temperature = *try_as_ref!(temperature as *const QO);
        let alpha = try_!(alpha.downcast_ref::<A>()).clone();

        super::make_private_quantile(
            input_domain, input_metric, candidates, temperature, alpha
        ).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let candidates = try_as_ref!(candidates);
    let alpha = try_as_ref!(alpha);

    let MI = input_metric.type_.clone();
    let TIA = try_!(input_domain.get_active_column_type());
    let QO = try_!(Type::try_from(QO));
    let A = try_!(Type::try_from(A));

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (TIA, @numbers),
        (QO, @floats),
        (A, @floats)
    ], (input_domain, input_metric, candidates, temperature, alpha))
}