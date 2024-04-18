use opendp_derive::bootstrap;
use polars_plan::dsl::Expr;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
    measures::MaxDivergence,
    metrics::PartitionDistance,
    transformations::traits::UnboundedMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
pub(crate) mod expr_laplace;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress))
)]
/// Create a differentially private measurement from an [`Expr`].
///
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `output_measure` - How to measure privacy loss.
/// * `expr` - The [`Expr`] to be privatized.
/// * `global_scale` - A tune-able parameter that affects the privacy-utility tradeoff.
pub fn make_private_expr<MI: 'static + Metric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    global_scale: f64,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (ExprDomain, MI): MetricSpace,
{
    expr.make_private(input_domain, input_metric, output_measure, global_scale)
}

pub trait PrivateExpr<MI: Metric, MO: Measure> {
    fn make_private(
        self,
        input_domain: ExprDomain,
        input_metric: MI,
        output_metric: MO,
        global_scale: f64,
    ) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>;
}

impl<M: UnboundedMetric + OuterMetric> PrivateExpr<PartitionDistance<M>, MaxDivergence<f64>>
    for Expr
{
    fn make_private(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<M>,
        _output_measure: MaxDivergence<f64>,
        global_scale: f64,
    ) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<M>, MaxDivergence<f64>>> {
        if expr_laplace::match_laplace(&self)?.is_some() {
            return expr_laplace::make_expr_laplace(input_domain, input_metric, self, global_scale);
        }

        match self {
            expr => fallible!(
                MakeMeasurement,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}
