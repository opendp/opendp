use crate::{
    core::{FfiResult, MetricSpace},
    domains::{ExprDomain, DataTypeFrom},
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast}, 
    transformations::{SumDatasetMetric, SumPrimitive, SumExprMetric}, 
    metrics::{IntDistance, L1, InsertDeleteDistance, SymmetricDistance},
    traits::{Number, ExactIntCast}
};
use crate::core::IntoAnyTransformationFfiResultExt;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_col_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<MI, QO>(input_domain: &AnyDomain, input_metric: &AnyMetric) -> FfiResult<*mut AnyTransformation>
        where MI: 'static + SumExprMetric<Distance = IntDistance> + Send + Sync,
        MI::InnerMetric: SumDatasetMetric,
        QO: Number + SumPrimitive + DataTypeFrom + ExactIntCast<i64>,
    
        (ExprDomain<MI::Context>, MI): MetricSpace,
        (ExprDomain<MI::Context>, MI::Context::SumMetric): MetricSpace {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<MI::Context>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        super::make_sum_expr(input_domain, input_metric).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);

    let MI = input_metric.type_.clone();
    let QO = try_!(input_domain.get_active_column_type());

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (QO, [u64, i64])
    ], (input_domain, input_metric))
}
