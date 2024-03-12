use crate::core::IntoAnyTransformationFfiResultExt;
use crate::{
    core::{FfiResult, MetricSpace},
    domains::{LazyFrameDomain, LazyGroupByDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::{InsertDeleteDistance, SymmetricDistance, L1},
    transformations::traits::UnboundedMetric,
};
#[no_mangle]
pub extern "C" fn opendp_transformations__make_group_by_stable(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    grouping_columns: *const AnyObject,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        grouping_columns: &AnyObject,
    ) -> Fallible<AnyTransformation>
    where
        M: UnboundedMetric + 'static + Send + Sync,
        (LazyFrameDomain, M): MetricSpace,
        (LazyGroupByDomain, L1<M>): MetricSpace,
    {
        let input_domain = try_!(input_domain.downcast_ref::<LazyFrameDomain>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        let grouping_columns =
            try_!(try_as_ref!(grouping_columns).downcast_ref::<Vec<String>>()).clone();
        super::make_group_by_stable(input_domain, input_metric, grouping_columns).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let grouping_columns = try_as_ref!(grouping_columns);

    let M = input_metric.type_.clone();

    dispatch!(
        monomorphize,
        [(M, [SymmetricDistance, InsertDeleteDistance])],
        (input_domain, input_metric, grouping_columns)
    )
    .into()
}
