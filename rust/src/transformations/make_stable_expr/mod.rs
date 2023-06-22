use opendp_derive::bootstrap;
use polars_plan::dsl::{AggExpr, Expr, FunctionExpr};

use crate::{
    core::{Metric, MetricSpace, Scalar, Transformation},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
    metrics::{InsertDeleteDistance, LpDistance, PartitionDistance, SymmetricDistance},
};

use super::{traits::UnboundedMetric, DatasetMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_col;

#[cfg(feature = "contrib")]
mod expr_clip;

#[cfg(feature = "contrib")]
mod expr_sum;

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

pub trait DatasetOuterMetric: OuterMetric {}
impl<M: DatasetMetric + OuterMetric> DatasetOuterMetric for M {}
impl DatasetOuterMetric for PartitionDistance<SymmetricDistance> {}

impl DatasetOuterMetric for PartitionDistance<InsertDeleteDistance> {}

impl<M: DatasetOuterMetric> StableExpr<M, M> for Expr
where
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
{
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: M,
    ) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>> {
        use Expr::*;
        use FunctionExpr::*;
        match self {
            #[cfg(feature = "contrib")]
            Expr::Column(_) => expr_col::make_expr_col(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Clip { .. },
                ..
            } => expr_clip::make_expr_clip(input_domain, input_metric, self),

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}

impl<MI: 'static + UnboundedMetric + DatasetOuterMetric, const P: usize>
    StableExpr<PartitionDistance<MI>, LpDistance<P, Scalar>> for Expr
where
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<MI>,
    ) -> Fallible<
        Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, Scalar>>,
    > {
        use Expr::*;
        match self {
            #[cfg(feature = "contrib")]
            Agg(AggExpr::Sum(_)) => {
                expr_sum::make_expr_sum::<_, _, P>(input_domain, input_metric, self.clone())
            }

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
        let pub_key_margin = Margin::new()
            .with_max_partition_length(3u32)
            .with_public_keys();
        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 2.5))?),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?
        .with_margin::<&str>(&[], pub_key_margin.clone())?
        .with_margin(
            &["A"],
            Margin::new()
                .with_public_keys()
                .with_max_partition_length(2u32),
        )?
        .with_margin(&["B"], Margin::new().with_max_partition_length(2u32))?
        .with_margin(&["C"], pub_key_margin.with_max_num_partitions(4))?;

        let lf = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)?
        .lazy();

        Ok((lf_domain, lf))
    }
}
