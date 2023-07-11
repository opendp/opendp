use polars::prelude::NumericNative;

use crate::{
    core::{FfiResult, MetricSpace, Domain},
    domains::{ExprDomain, NumericDataType, OuterMetric},
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast, AnyObject}, 
    metrics::{L1, InsertDeleteDistance, SymmetricDistance},
    traits::{TotalOrd, CheckAtom}
};
use crate::core::IntoAnyTransformationFfiResultExt;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_clamp_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    bounds: *const AnyObject
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<M, TA>(input_domain: &AnyDomain, input_metric: &AnyMetric, bounds: &AnyObject) -> FfiResult<*mut AnyTransformation>
    where 
        TA: 'static + Clone + TotalOrd + CheckAtom + NumericNative + NumericDataType,
        <ExprDomain<M::LazyDomain> as Domain>::Carrier: Send + Sync,
        M: OuterMetric,
        M::Distance: 'static + Clone + Send + Sync,
        (ExprDomain<M::LazyDomain>, M): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<M::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        let bounds = try_!(bounds.downcast_ref::<(TA, TA)>()).clone();
        super::make_clamp_expr(input_domain, input_metric, bounds).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let bounds = try_as_ref!(bounds);

    let M = input_metric.type_.clone();
    let TA = try_!(input_domain.get_active_column_type());

    dispatch!(monomorphize, [
        (M, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (TA, @numbers)
    ], (input_domain, input_metric, bounds))
}
