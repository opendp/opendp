use opendp_derive::bootstrap;
use polars_plan::dsl::{AggExpr, Expr};

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::{ExprDomain, OuterMetric},
    error::Fallible,
    measures::MaxDivergence,
    metrics::PartitionDistance,
    transformations::{traits::UnboundedMetric, DatasetOuterMetric},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(feature = "contrib")]
mod expr_count;

#[cfg(feature = "contrib")]
pub(crate) mod expr_laplace;

#[cfg(feature = "contrib")]
mod expr_literal;

#[cfg(feature = "contrib")]
mod expr_postprocess;

#[cfg(feature = "contrib")]
pub(crate) mod expr_report_noisy_max_gumbel;

#[bootstrap(
    features("contrib"),
    arguments(
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
        param(rust_type = "Option<f64>", c_type = "AnyObject *", default = b"null")
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
/// * `param` - A tune-able parameter that affects the privacy-utility tradeoff.
pub fn make_private_expr<MI: 'static + Metric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    param: f64,
) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (ExprDomain, MI): MetricSpace,
{
    expr.make_private(input_domain, input_metric, output_measure, param)
}

pub trait PrivateExpr<MI: Metric, MO: Measure> {
    fn make_private(
        self,
        input_domain: ExprDomain,
        input_metric: MI,
        output_metric: MO,
        param: f64,
    ) -> Fallible<Measurement<ExprDomain, Expr, MI, MO>>;
}

impl<M: UnboundedMetric + OuterMetric> PrivateExpr<PartitionDistance<M>, MaxDivergence<f64>>
    for Expr
where
    PartitionDistance<M>: DatasetOuterMetric,
    <PartitionDistance<M> as Metric>::Distance: Clone,
    (ExprDomain, PartitionDistance<M>): MetricSpace,
{
    fn make_private(
        self,
        input_domain: ExprDomain,
        input_metric: PartitionDistance<M>,
        output_measure: MaxDivergence<f64>,
        param: f64,
    ) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<M>, MaxDivergence<f64>>> {
        if expr_laplace::match_laplace(&self)?.is_some() {
            return expr_laplace::make_expr_laplace(input_domain, input_metric, self, param);
        }

        if expr_report_noisy_max_gumbel::match_rnm_gumbel(&self)?.is_some() {
            return expr_report_noisy_max_gumbel::make_expr_report_noisy_max_gumbel::<
                PartitionDistance<M>,
            >(input_domain, input_metric, self, param);
        }

        match self {
            #[cfg(feature = "contrib")]
            Expr::KeepName(expr) => {
                expr_postprocess::make_expr_postprocess(
                    input_domain,
                    input_metric,
                    output_measure,
                    vec![*expr],
                    move |exprs| {
                        let [expr] = <[Expr; 1]>::try_from(exprs)
                            .expect("Will always have exactly one expression.");
                        Ok(expr.name().keep())
                    },
                    param
                )
            }
            #[cfg(feature = "contrib")]
            Expr::Alias(expr, name) => {
                expr_postprocess::make_expr_postprocess(
                    input_domain,
                    input_metric,
                    output_measure,
                    vec![*expr],
                    move |exprs| {
                        let [expr] = <[Expr; 1]>::try_from(exprs)
                            .expect("Will always have exactly one expression.");
                        Ok(expr.alias(name.as_ref()))
                    },
                    param
                )
            }

            #[cfg(feature = "contrib")]
            Expr::BinaryExpr { left, op, right } => {
                expr_postprocess::make_expr_postprocess(
                    input_domain,
                    input_metric,
                    output_measure,
                    vec![*left, *right],
                    move |exprs| {
                        let [left, right] = <[Expr; 2]>::try_from(exprs)
                            .expect("Will always have exactly two expressions.")
                            .map(Box::new);
                        Ok(Expr::BinaryExpr { left, op, right })
                    },
                    param
                )
            }

            #[cfg(feature = "contrib")]
            Expr::Agg(AggExpr::Count(_, _)) => {
                expr_count::make_expr_private_count(input_domain, input_metric, self.clone())
            }

            #[cfg(feature = "contrib")]
            Expr::Gather { expr, idx, returns_scalar } => {
                expr_postprocess::make_expr_postprocess(
                    input_domain,
                    input_metric,
                    output_measure,
                    vec![*expr, *idx],
                    move |exprs| {
                        let [expr, idx] = <[Expr; 2]>::try_from(exprs)
                            .expect("Will always have exactly two expressions.")
                            .map(Box::new);
                        Ok(Expr::Gather { expr, idx, returns_scalar })
                    },
                    param
                )
            }

            #[cfg(feature = "contrib")]
            Expr::Literal(_) => {
                expr_literal::make_expr_private_lit(input_domain, input_metric, self.clone())
            }

            expr => fallible!(
                MakeMeasurement,
                "Expr is not recognized at this time: {:?}. If you would like to see this supported, please file an issue.",
                expr
            )
        }
    }
}
