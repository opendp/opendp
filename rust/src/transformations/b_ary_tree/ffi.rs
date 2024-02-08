use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
        util::Type,
    },
    metrics::{L1Distance, L2Distance},
    traits::{Integer, Number},
    transformations::{make_b_ary_tree, BAryTreeMetric},
};

use super::choose_branching_factor;

#[no_mangle]
pub extern "C" fn opendp_transformations__make_b_ary_tree(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    leaf_count: u32,
    branching_factor: u32,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<Q>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        leaf_count: u32,
        branching_factor: u32,
        M: Type,
        TA: Type,
    ) -> Fallible<AnyTransformation>
    where
        Q: Number,
    {
        fn monomorphize2<M, TA>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            leaf_count: u32,
            branching_factor: u32,
        ) -> Fallible<AnyTransformation>
        where
            TA: Integer,
            (VectorDomain<AtomDomain<TA>>, M): MetricSpace,
            M: 'static + BAryTreeMetric,
            M::Distance: Number,
        {
            let input_domain = input_domain
                .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
                .clone();
            let input_metric = input_metric.downcast_ref::<M>()?.clone();
            make_b_ary_tree::<M, TA>(input_domain, input_metric, leaf_count, branching_factor)
                .into_any()
        }

        dispatch!(monomorphize2, [
            (M, [L1Distance<Q>, L2Distance<Q>]),
            (TA, @integers)
        ], (input_domain, input_metric, leaf_count, branching_factor))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let M = input_metric.type_.clone();
    let TA = try_!(input_domain.type_.get_atom());
    let Q = try_!(M.get_atom());
    dispatch!(monomorphize, [
        (Q, @integers)
    ], (input_domain, input_metric, leaf_count, branching_factor, M, TA))
    .into()
}

#[no_mangle]
pub extern "C" fn opendp_transformations__choose_branching_factor(size_guess: u32) -> u32 {
    choose_branching_factor(size_guess)
}
