use std::ffi::c_double;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    traits::Number,
    transformations::{make_quantile_score_candidates, traits::UnboundedMetric},
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_quantile_score_candidates(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    candidates: *const AnyObject,
    alpha: c_double,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let candidates = try_as_ref!(candidates);
    let alpha = alpha;

    fn monomorphize<M, TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        candidates: &AnyObject,
        alpha: f64,
    ) -> Fallible<AnyTransformation>
    where
        M: 'static + UnboundedMetric,
        TIA: 'static + Number,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<M>()?.clone();
        let candidates = candidates.downcast_ref::<Vec<TIA>>()?.clone();
        make_quantile_score_candidates::<M, TIA>(input_domain, input_metric, candidates, alpha)
            .into_any()
    }
    let M = input_metric.type_.clone();
    let TIA = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (M, @dataset_metrics),
        (TIA, @numbers)
    ], (input_domain, input_metric, candidates, alpha))
    .into()
}
