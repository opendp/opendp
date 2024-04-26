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
pub mod test_helper;
