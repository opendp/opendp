use std::ffi::c_double;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
    metrics::SymmetricDistance,
    traits::Number,
    transformations::make_quantile_score_candidates,
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

    fn monomorphize<TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        candidates: &AnyObject,
        alpha: f64,
    ) -> Fallible<AnyTransformation>
    where
        TIA: 'static + Number,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
        let candidates = candidates.downcast_ref::<Vec<TIA>>()?.clone();
        make_quantile_score_candidates::<TIA>(input_domain, input_metric, candidates, alpha)
            .into_any()
    }
    let TIA = try_!(input_domain.type_.get_atom());
    dispatch!(monomorphize, [
        (TIA, @numbers)
    ], (input_domain, input_metric, candidates, alpha))
    .into()
}
