use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
    metrics::{InsertDeleteDistance, SymmetricDistance},
    transformations::{MakeSum, make_sum},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_sum(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<MI, T>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        MI: 'static + Metric,
        T: 'static + MakeSum<MI>,
        (VectorDomain<AtomDomain<T>>, MI): MetricSpace,
    {
        make_sum::<MI, T>(
            input_domain
                .downcast_ref::<VectorDomain<AtomDomain<T>>>()?
                .clone(),
            input_metric.downcast_ref::<MI>()?.clone(),
        )
        .into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let MI = input_metric.type_.clone();
    let T = try_!(input_domain.type_.get_atom());
    dispatch!(
        monomorphize,
        [
            (MI, [SymmetricDistance, InsertDeleteDistance]),
            (T, [u32, u64, u128, i32, i64, i128, f32, f64])
        ],
        (input_domain, input_metric)
    )
    .into()
}
