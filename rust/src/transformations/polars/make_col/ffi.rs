use std::ffi::c_char;

use crate::core::{IntoAnyTransformationFfiResultExt, Domain};
use crate::domains::OuterMetric;
use crate::ffi::util::to_str;
use crate::{
    core::{FfiResult, MetricSpace},
    domains::ExprDomain,
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
    error::Fallible,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_col_expr(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    col_name: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        col_name: String,
    ) -> Fallible<AnyTransformation>
    where
        M: OuterMetric,
        <M::LazyDomain as Domain>::Carrier: Send + Sync,
        M::Distance: 'static + Clone + Send + Sync,
        (ExprDomain<M::LazyDomain>, M): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<ExprDomain<M::LazyDomain>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        super::make_col(input_domain, input_metric, col_name).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let col_name = try_!(to_str(col_name)).to_string();

    let MI = input_metric.type_.clone();

    dispatch!(monomorphize, [
        (MI, [L1<SymmetricDistance>, L1<InsertDeleteDistance>, SymmetricDistance, InsertDeleteDistance])
    ], (input_domain, input_metric, col_name)).into()
}
