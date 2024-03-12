use crate::core::IntoAnyTransformationFfiResultExt;
use crate::{
    core::{Domain, FfiResult, MetricSpace},
    domains::{ExprDomain, NumericDataType},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
    traits::{ExactIntCast, Number},
    transformations::{traits::UnboundedMetric, SumOuterMetric, Summand},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_sum_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        MI: 'static + SumOuterMetric<QO>,
        MI::InnerMetric: UnboundedMetric,
        QO: Number + Summand + NumericDataType + ExactIntCast<i64>,
        <ExprDomain<MI::LazyDomain> as Domain>::Carrier: Send + Sync,

        (ExprDomain<MI::LazyDomain>, MI): MetricSpace,
        (ExprDomain<MI::LazyDomain>, MI::SumMetric): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<MI::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<MI>()).clone();
        super::make_sum_expr(input_domain, input_metric).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);

    let MI = input_metric.type_.clone();
    let TI = try_!(input_domain.get_active_column_type());

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance]), 
        (TI, [u32, u64, i8, i16, i32, i64, f64, f32])
    ], (input_domain, input_metric)).into()
}
