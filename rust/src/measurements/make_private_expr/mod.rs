use opendp_derive::bootstrap;
use polars_plan::dsl::Expr;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::ExprDomain,
    error::Fallible,
    measures::MaxDivergence,
    metrics::PartitionDistance,
    transformations::traits::UnboundedMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_len;

#[cfg(feature = "contrib")]
pub(crate) mod expr_index_candidates;

#[cfg(feature = "contrib")]
pub(crate) mod expr_laplace;

#[cfg(feature = "contrib")]
mod expr_literal;

#[cfg(feature = "contrib")]
mod expr_postprocess;

#[cfg(feature = "contrib")]
pub(crate) mod expr_report_noisy_max_gumbel;

#[bootstrap(
    features("contrib", "honest-but-curious"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        global_scale(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress))
)]
/// Create a differentially private measurement from an [`Expr`].
///
/// # Features
/// * `honest-but-curious` - The privacy guarantee governs only at most one evaluation of the released expression.
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
    global_scale: Option<f64>,
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
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>;
}

impl<M: 'static + UnboundedMetric> PrivateExpr<PartitionDistance<M>, MaxDivergence<f64>> for Expr {
    fn make_private(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<M>,
        output_measure: MaxDivergence<f64>,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<M>, MaxDivergence<f64>>> {
        if expr_laplace::match_laplace(&self)?.is_some() {
            return expr_laplace::make_expr_laplace(input_domain, input_metric, self, global_scale);
        }

        if expr_report_noisy_max_gumbel::match_report_noisy_max_gumbel(&self)?.is_some() {
            return expr_report_noisy_max_gumbel::make_expr_report_noisy_max_gumbel::<
                PartitionDistance<M>,
            >(input_domain, input_metric, self, global_scale);
        }

        if expr_index_candidates::match_index_candidates(&self)?.is_some() {
            return expr_index_candidates::make_expr_index_candidates::<PartitionDistance<M>, _>(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }

        if let Some(meas) = expr_postprocess::match_postprocess(
            input_domain.clone(),
            input_metric.clone(),
            output_measure.clone(),
            self.clone(),
            global_scale,
        )? {
            return Ok(meas);
        }

        match self {
            #[cfg(feature = "contrib")]
            Expr::Len => {
                expr_len::make_expr_private_len(input_domain, input_metric, output_measure, self.clone())
            }

            #[cfg(feature = "contrib")]
            Expr::Literal(_) => {
                expr_literal::make_expr_private_lit(input_domain, input_metric, self.clone())
            }

            expr => fallible!(
                MakeMeasurement,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            ),
        }
    }
}
