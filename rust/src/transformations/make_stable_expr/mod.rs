use opendp_derive::bootstrap;
use polars_plan::dsl::Expr;

use crate::{
    core::{Metric, MetricSpace, Transformation},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
};

use super::DatasetMetric;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_col;

#[bootstrap(
    features("contrib"),
    arguments(output_metric(c_type = "AnyMetric *", rust_type = b"null")),
    generics(MI(suppress), MO(suppress))
)]
/// Create a stable transformation from an [`Expr`].
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `expr` - The [`Expr`] to be privatized.
pub fn make_stable_expr<MI: 'static + Metric, MO: 'static + Metric>(
    input_domain: ExprDomain,
    input_metric: MI,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, MI, MO>>
where
    Expr: StableExpr<MI, MO>,
    (ExprDomain, MI): MetricSpace,
    (ExprDomain, MO): MetricSpace,
{
    expr.make_stable(input_domain, input_metric)
}

pub trait StableExpr<MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<ExprDomain, ExprDomain, MI, MO>>;
}

impl<M: OuterMetric> StableExpr<M, M> for Expr
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
{
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: M,
    ) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>> {
        use Expr::*;
        match self {
            #[cfg(feature = "contrib")]
            Column(_) => expr_col::make_expr_col(input_domain, input_metric, self),
            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}

#[cfg(test)]
pub mod polars_test {

    use crate::domains::{AtomDomain, LazyFrameDomain, Margin, SeriesDomain};
    use crate::error::*;
    use polars::prelude::*;

    pub fn get_test_data() -> Fallible<(LazyFrameDomain, LazyFrame)> {
        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("chunk_2_bool", AtomDomain::<bool>::default()),
            SeriesDomain::new("cycle_5_alpha", AtomDomain::<String>::default()),
            SeriesDomain::new("const_1f64", AtomDomain::<f64>::default()),
            SeriesDomain::new("chunk_(..10u32)", AtomDomain::<u32>::default()),
            SeriesDomain::new("cycle_(..100i32)", AtomDomain::<i32>::default()),
        ])?
        .with_margin::<&str>(
            &[],
            Margin::new()
                .with_public_lengths()
                .with_max_partition_length(1000),
        )?
        .with_margin(
            &["chunk_2_bool"],
            Margin::new()
                .with_public_lengths()
                .with_max_partition_length(500)
                .with_max_num_partitions(2)
                .with_max_partition_contributions(1),
        )?
        .with_margin(
            &["chunk_2_bool", "cycle_5_alpha"],
            Margin::new()
                .with_public_keys()
                .with_max_partition_length(200),
        )?
        .with_margin(
            &["chunk_(..10u32)"],
            Margin::new()
                .with_public_keys()
                .with_max_partition_length(100),
        )?;

        let lf = df!(
            "chunk_2_bool" => [[false; 500], [true; 500]].concat(),
            "cycle_5_alpha" => ["A", "B", "C", "D", "E"].repeat(200),
            "const_1f64" => [1.0; 1000],
            "chunk_(..10u32)" => (0..10u32).flat_map(|i| [i; 100].into_iter()).collect::<Vec<_>>(),
            "cycle_(..100i32)" => (0..100i32).cycle().take(1000).collect::<Vec<_>>()
        )?
        .lazy();

        Ok((lf_domain, lf))
    }
}
