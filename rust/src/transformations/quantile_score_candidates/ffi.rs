use std::ffi::c_char;

use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, Downcast},
        util::Type,
    },
    traits::Number,
    transformations::{
        make_quantile_score_candidates, quantile_score_candidates::IntoFrac, ARDatasetMetric,
    },
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_quantile_score_candidates(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    candidates: *const AnyObject,
    alpha: *const AnyObject,
    F: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let candidates = try_as_ref!(candidates);
    let alpha = try_as_ref!(alpha);

    fn monomorphize<TIA, F, M>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        candidates: &AnyObject,
        alpha: &AnyObject,
    ) -> FfiResult<*mut AnyTransformation>
    where
        TIA: 'static + Number,
        F: 'static + IntoFrac + Clone,
        M: 'static + ARDatasetMetric + Send + Sync,
        (VectorDomain<AtomDomain<TIA>>, M): MetricSpace,
    {
        let input_domain =
            try_!(input_domain.downcast_ref::<VectorDomain<AtomDomain<TIA>>>()).clone();
        let input_metric = try_!(input_metric.downcast_ref::<M>()).clone();
        let candidates = try_!(candidates.downcast_ref::<Vec<TIA>>()).clone();
        let alpha = try_!(alpha.downcast_ref::<F>()).clone();
        make_quantile_score_candidates::<TIA, F, M>(input_domain, input_metric, candidates, alpha)
            .into_any()
    }
    let TIA = try_!(input_domain.type_.get_atom());
    let F = try_!(Type::try_from(F));
    let M = input_metric.type_.clone();
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (F, [f32, f64, (usize, usize), (i32, i32)]),
        (M, @dataset_metrics)
    ], (input_domain, input_metric, candidates, alpha))
}
