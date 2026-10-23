use crate::{
    core::{FfiResult, IntoAnyTransformationFfiResultExt, Metric, MetricSpace},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyTransformation, Downcast},
        util::Type,
    },
    metrics::{L1Distance, L2Distance},
    traits::{InfCast, Integer, Number},
    transformations::{BAryTreeMetric, make_b_ary_tree},
};

use super::choose_branching_factor;

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_b_ary_tree(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    output_metric: *const AnyMetric,
    leaf_count: u32,
    branching_factor: u32,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QI>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        output_metric: &AnyMetric,
        leaf_count: u32,
        branching_factor: u32,
        MO: Type,
        TA: Type,
        QI: Type,
    ) -> Fallible<AnyTransformation>
    where
        QI: Number,
        f64: InfCast<QI>,
    {
        fn monomorphize2<MI, MO, TA>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            output_metric: &AnyMetric,
            leaf_count: u32,
            branching_factor: u32,
        ) -> Fallible<AnyTransformation>
        where
            TA: Integer,
            MI: 'static + Metric,
            MI::Distance: Number,
            MO: 'static + BAryTreeMetric<MI>,
            MO::Distance: Number + InfCast<MI::Distance>,
            (VectorDomain<AtomDomain<TA>>, MI): MetricSpace,
            (VectorDomain<AtomDomain<TA>>, MO): MetricSpace,
        {
            let input_domain = input_domain
                .downcast_ref::<VectorDomain<AtomDomain<TA>>>()?
                .clone();
            let input_metric = input_metric.downcast_ref::<MI>()?.clone();
            let output_metric = output_metric.downcast_ref::<MO>()?.clone();
            make_b_ary_tree(
                input_domain,
                input_metric,
                output_metric,
                leaf_count,
                branching_factor,
            )
            .into_any()
        }

        let MI = input_metric.type_.clone();
        dispatch!(monomorphize2, [
            (MI, [L1Distance<QI>]),
            (MO, [L1Distance<QI>, L2Distance<f64>]),
            (TA, @integers)
        ], (input_domain, input_metric, output_metric, leaf_count, branching_factor))
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let output_metric = try_as_ref!(output_metric);
    let MO = output_metric.type_.clone();
    let TA = try_!(input_domain.type_.get_atom());
    let QI = try_!(input_metric.type_.get_atom());
    dispatch!(monomorphize, [
        (QI, @integers)
    ], (input_domain, input_metric, output_metric, leaf_count, branching_factor, MO, TA, QI))
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__choose_branching_factor(size_guess: u32) -> u32 {
    choose_branching_factor(size_guess)
}
