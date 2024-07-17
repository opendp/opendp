use opendp_derive::bootstrap;
use polars_plan::dsl::{AggExpr, Expr, FunctionExpr};

use crate::{
    core::{Metric, MetricSpace, Transformation},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
    metrics::{LInfDistance, LpDistance, Parallel, PartitionDistance},
    polars::get_disabled_features_message,
};

use super::{traits::UnboundedMetric, DatasetMetric};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_alias;

#[cfg(feature = "contrib")]
mod expr_binary;

#[cfg(feature = "contrib")]
mod expr_boolean_function;

#[cfg(feature = "contrib")]
mod expr_clip;

#[cfg(feature = "contrib")]
mod expr_col;

#[cfg(feature = "contrib")]
pub(crate) mod expr_discrete_quantile_score;

#[cfg(feature = "contrib")]
mod expr_fill_nan;

#[cfg(feature = "contrib")]
mod expr_fill_null;

#[cfg(feature = "contrib")]
mod expr_len;

#[cfg(feature = "contrib")]
mod expr_lit;

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
        if expr_fill_nan::match_fill_nan(&self).is_some() {
            return expr_fill_nan::make_expr_fill_nan(input_domain, input_metric, self);
        }

        use Expr::*;
        use FunctionExpr::*;
        match self {

            #[cfg(feature = "contrib")]
            Alias(_, _) => expr_alias::make_expr_alias(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Expr::BinaryExpr { .. } => expr_binary::make_expr_binary(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Boolean(_),
                ..
            } => return expr_boolean_function::make_expr_boolean_function(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Clip { .. },
                ..
            } => expr_clip::make_expr_clip(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: FillNull { .. },
                ..
            } => expr_fill_null::make_expr_fill_null(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Column(_) => expr_col::make_expr_col(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Literal(_) => expr_lit::make_expr_lit(input_domain, input_metric, self),

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                expr,
                get_disabled_features_message()
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

            #[cfg(feature = "contrib")]
            Len => expr_len::make_expr_len(input_domain, input_metric, self),

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                expr,
                get_disabled_features_message()
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
        if expr_discrete_quantile_score::match_discrete_quantile_score(&self)?.is_some() {
            return expr_discrete_quantile_score::make_expr_discrete_quantile_score(
                input_domain,
                input_metric,
                self,
            );
        }
        match self {
            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. {:?}If you would like to see this supported, please file an issue.",
                expr,
                get_disabled_features_message()
            )
        }
    }
}

#[cfg(test)]
pub mod test_helper;
