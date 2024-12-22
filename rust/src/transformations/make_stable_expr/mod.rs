use opendp_derive::bootstrap;
use polars_plan::dsl::{AggExpr, Expr, FunctionExpr};

use crate::{
    core::{Metric, MetricSpace, Transformation},
    domains::{ExprDomain, OuterMetric, WildExprDomain},
    error::Fallible,
    metrics::{LInfDistance, LpDistance, Parallel, PartitionDistance},
    polars::get_disabled_features_message,
};

use super::traits::UnboundedMetric;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_alias;

#[cfg(feature = "contrib")]
mod expr_binary;

#[cfg(feature = "contrib")]
mod expr_boolean_function;

#[cfg(feature = "contrib")]
mod expr_cast;

#[cfg(feature = "contrib")]
mod expr_clip;

#[cfg(feature = "contrib")]
mod expr_col;

#[cfg(feature = "contrib")]
mod expr_count;

#[cfg(feature = "contrib")]
mod expr_cut;

#[cfg(feature = "contrib")]
pub(crate) mod expr_discrete_quantile_score;

#[cfg(feature = "contrib")]
pub(crate) mod expr_drop_nan_or_null;

#[cfg(feature = "contrib")]
mod expr_fill_nan;

#[cfg(feature = "contrib")]
mod expr_fill_null;

#[cfg(feature = "contrib")]
mod expr_filter;

#[cfg(feature = "contrib")]
mod expr_len;

#[cfg(feature = "contrib")]
mod expr_lit;

#[cfg(feature = "contrib")]
pub(crate) mod expr_replace;

#[cfg(feature = "contrib")]
mod expr_replace_strict;

#[cfg(feature = "contrib")]
mod expr_sum;

#[cfg(feature = "contrib")]
mod expr_to_physical;

#[cfg(feature = "contrib")]
mod namespace_dt;

#[cfg(feature = "contrib")]
mod namespace_str;

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
    input_domain: WildExprDomain,
    input_metric: MI,
    expr: Expr,
) -> Fallible<Transformation<WildExprDomain, ExprDomain, MI, MO>>
where
    Expr: StableExpr<MI, MO>,
    (WildExprDomain, MI): MetricSpace,
    (ExprDomain, MO): MetricSpace,
{
    expr.make_stable(input_domain, input_metric)
}

pub trait StableExpr<MI: Metric, MO: Metric> {
    fn make_stable(
        self,
        input_domain: WildExprDomain,
        input_metric: MI,
    ) -> Fallible<Transformation<WildExprDomain, ExprDomain, MI, MO>>;
}

impl<M: OuterMetric> StableExpr<M, M> for Expr
where
    M::InnerMetric: UnboundedMetric,
    M::Distance: Clone,
    (WildExprDomain, M): MetricSpace,
    (ExprDomain, M): MetricSpace,
{
    fn make_stable(
        self,
        input_domain: WildExprDomain,
        input_metric: M,
    ) -> Fallible<Transformation<WildExprDomain, ExprDomain, M, M>> {
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
            Cast { .. } => expr_cast::make_expr_cast(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Clip { .. },
                ..
            } => expr_clip::make_expr_clip(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: DropNans | DropNulls,
                ..
            } => expr_drop_nan_or_null::make_expr_drop_nan_or_null(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: FillNull { .. },
                ..
            } => expr_fill_null::make_expr_fill_null(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Filter { .. } => expr_filter::make_expr_filter(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Column(_) => expr_col::make_expr_col(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Cut { .. },
                ..
            } => expr_cut::make_expr_cut(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Literal(_) => expr_lit::make_expr_lit(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: ToPhysical,
                ..
            } => expr_to_physical::make_expr_to_physical(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: Replace,
                ..
            } => expr_replace::make_expr_replace(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: ReplaceStrict { .. },
                ..
            } => expr_replace_strict::make_expr_replace_strict(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: FunctionExpr::TemporalExpr(_),
                ..
            } => namespace_dt::make_namespace_dt(input_domain, input_metric, self),

            #[cfg(feature = "contrib")]
            Function {
                function: FunctionExpr::StringExpr(_),
                ..
            } => namespace_str::make_namespace_str(input_domain, input_metric, self),

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
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
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<MI>,
    ) -> Fallible<
        Transformation<WildExprDomain, ExprDomain, PartitionDistance<MI>, LpDistance<P, f64>>,
    > {
        use Expr::*;
        match self {
            #[cfg(feature = "contrib")]
            Agg(AggExpr::Count(_, _) | AggExpr::NUnique(_)) | Function { function: FunctionExpr::NullCount, .. } => {
                expr_count::make_expr_count(input_domain, input_metric, self)
            }

            #[cfg(feature = "contrib")]
            Agg(AggExpr::Sum(_)) => {
                expr_sum::make_expr_sum(input_domain, input_metric, self)
            }

            #[cfg(feature = "contrib")]
            Len => expr_len::make_expr_len(input_domain, input_metric, self),

            expr => fallible!(
                MakeTransformation,
                "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
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
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<MI>,
    ) -> Fallible<
        Transformation<
            WildExprDomain,
            ExprDomain,
            PartitionDistance<MI>,
            Parallel<LInfDistance<f64>>,
        >,
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
                "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
                expr,
                get_disabled_features_message()
            )
        }
    }
}

#[cfg(test)]
pub mod test_helper;
