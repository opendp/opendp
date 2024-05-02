use opendp_derive::bootstrap;
use polars_plan::dsl::{AggExpr, Expr, FunctionExpr};

use crate::{
    core::{Metric, MetricSpace, Transformation},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
    metrics::{LInfDistance, LpDistance, Parallel, PartitionDistance},
};

use super::{traits::UnboundedMetric, DatasetMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_clip;

#[cfg(feature = "contrib")]
mod expr_col;

#[cfg(feature = "contrib")]
pub(crate) mod expr_discrete_quantile_score;

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
/// * `expr` - The expression to be analyzed for stability.
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
        use FunctionExpr::*;
        match self {

            #[cfg(feature = "contrib")]
            Function {
                function: Clip { .. },
                ..
            } => expr_clip::make_expr_clip(input_domain, input_metric, self),

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

impl<MI, const P: usize> StableExpr<PartitionDistance<MI>, LpDistance<P, f64>> for Expr
where
    MI: 'static + UnboundedMetric,
{
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<MI>,
    ) -> Fallible<Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>>
    {
        use Expr::*;
        match self {
            #[cfg(feature = "contrib")]
            Agg(AggExpr::Sum(_)) => {
                expr_sum::make_expr_sum(input_domain, input_metric, self)
            }

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}

impl<MI> StableExpr<PartitionDistance<MI>, Parallel<LInfDistance<f64>>> for Expr
where
    MI: 'static + UnboundedMetric,
{
    fn make_stable(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<MI>,
    ) -> Fallible<
        Transformation<ExprDomain, ExprDomain, PartitionDistance<MI>, Parallel<LInfDistance<f64>>>,
    > {
        if expr_discrete_quantile_score::match_dq_score(&self)?.is_some() {
            return expr_discrete_quantile_score::make_expr_discrete_quantile_score(
                input_domain,
                input_metric,
                self,
            );
        }
        match self {
            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}

#[cfg(test)]
pub mod test_helper;
