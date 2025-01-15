use opendp_derive::bootstrap;
use polars_plan::dsl::Expr;

use crate::{
    combinators::{make_approximate, BasicCompositionMeasure},
    core::{Measure, Measurement, Metric, MetricSpace, Transformation},
    domains::{Context, ExprDomain, ExprPlan, MarginPub, WildExprDomain},
    error::Fallible,
    measures::{Approximate, MaxDivergence, ZeroConcentratedDivergence},
    metrics::PartitionDistance,
    polars::get_disabled_features_message,
    transformations::traits::UnboundedMetric,
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[cfg(feature = "contrib")]
mod expr_len;

#[cfg(feature = "contrib")]
pub(crate) mod expr_index_candidates;

#[cfg(feature = "contrib")]
pub(crate) mod expr_noise;

#[cfg(feature = "contrib")]
mod expr_literal;

#[cfg(feature = "contrib")]
mod expr_postprocess;

#[cfg(feature = "contrib")]
pub(crate) mod expr_report_noisy_max;

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
/// # Arguments
/// * `input_domain` - The domain of the input data.
/// * `input_metric` - How to measure distances between neighboring input data sets.
/// * `output_measure` - How to measure privacy loss.
/// * `expr` - The [`Expr`] to be privatized.
/// * `global_scale` - A tune-able parameter that affects the privacy-utility tradeoff.
///
/// # Why honest-but-curious?
/// The privacy guarantee governs only at most one evaluation of the released expression.
pub fn make_private_expr<MI: 'static + Metric, MO: 'static + Measure>(
    input_domain: WildExprDomain,
    input_metric: MI,
    output_measure: MO,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, MI, MO>>
where
    Expr: PrivateExpr<MI, MO>,
    (WildExprDomain, MI): MetricSpace,
{
    expr.make_private(input_domain, input_metric, output_measure, global_scale)
}

pub trait PrivateExpr<MI: Metric, MO: Measure> {
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: MI,
        output_metric: MO,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<WildExprDomain, ExprPlan, MI, MO>>;
}

impl<M: 'static + UnboundedMetric> PrivateExpr<PartitionDistance<M>, MaxDivergence> for Expr {
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<M>,
        output_measure: MaxDivergence,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<WildExprDomain, ExprPlan, PartitionDistance<M>, MaxDivergence>> {
        if expr_noise::match_noise_shim(&self)?.is_some() {
            return expr_noise::make_expr_noise(input_domain, input_metric, self, global_scale);
        }

        if expr_report_noisy_max::match_report_noisy_max(&self)?.is_some() {
            return expr_report_noisy_max::make_expr_report_noisy_max::<M>(
                input_domain,
                input_metric,
                self,
                global_scale,
            );
        }

        make_private_measure_agnostic(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
        )
    }
}

impl<M: 'static + UnboundedMetric> PrivateExpr<PartitionDistance<M>, ZeroConcentratedDivergence>
    for Expr
{
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<M>,
        output_measure: ZeroConcentratedDivergence,
        global_scale: Option<f64>,
    ) -> Fallible<
        Measurement<WildExprDomain, ExprPlan, PartitionDistance<M>, ZeroConcentratedDivergence>,
    > {
        if expr_noise::match_noise_shim(&self)?.is_some() {
            return expr_noise::make_expr_noise(input_domain, input_metric, self, global_scale);
        }

        make_private_measure_agnostic(
            input_domain,
            input_metric,
            output_measure,
            self,
            global_scale,
        )
    }
}

impl<MI: 'static + UnboundedMetric, MO: 'static + Measure>
    PrivateExpr<PartitionDistance<MI>, Approximate<MO>> for Expr
where
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
{
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: PartitionDistance<MI>,
        output_measure: Approximate<MO>,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<WildExprDomain, ExprPlan, PartitionDistance<MI>, Approximate<MO>>>
    {
        make_approximate(self.make_private(
            input_domain,
            input_metric,
            output_measure.0,
            global_scale,
        )?)
    }
}

fn make_private_measure_agnostic<
    MI: 'static + UnboundedMetric,
    MO: 'static + BasicCompositionMeasure<Distance = f64>,
>(
    input_domain: WildExprDomain,
    input_metric: PartitionDistance<MI>,
    output_measure: MO,
    expr: Expr,
    global_scale: Option<f64>,
) -> Fallible<Measurement<WildExprDomain, ExprPlan, PartitionDistance<MI>, MO>>
where
    Expr: PrivateExpr<PartitionDistance<MI>, MO>,
{
    if expr_index_candidates::match_index_candidates(&expr)?.is_some() {
        return expr_index_candidates::make_expr_index_candidates::<PartitionDistance<MI>, _>(
            input_domain,
            input_metric,
            output_measure,
            expr,
            global_scale,
        );
    }

    if let Some(meas) = expr_postprocess::match_postprocess(
        input_domain.clone(),
        input_metric.clone(),
        output_measure.clone(),
        expr.clone(),
        global_scale,
    )? {
        return Ok(meas);
    }

    match &expr {
        #[cfg(feature = "contrib")]
        Expr::Len => {
            expr_len::make_expr_private_len(input_domain, input_metric, output_measure, expr)
        }

        #[cfg(feature = "contrib")]
        Expr::Literal(_) => {
            expr_literal::make_expr_private_lit(input_domain, input_metric, expr)
        }

        expr => fallible!(
            MakeMeasurement,
            "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
            expr,
            get_disabled_features_message()
        ),
    }
}

/// Approximate the c-stability of a transformation.
///
/// See section 2.3 in [Privacy Integrated Queries, Frank McSherry](https://css.csail.mit.edu/6.5660/2024/readings/pinq.pdf)
/// Since OpenDP uses privacy maps, the stability constant may vary as d_in is varied.
///
/// This function only approximates the stability constant `c`, when...
/// * one row is added/removed in unbounded-DP
/// * one row is changed in bounded-DP
pub(crate) fn approximate_c_stability<MI: UnboundedMetric, MO: Metric>(
    trans: &Transformation<WildExprDomain, ExprDomain, PartitionDistance<MI>, MO>,
) -> Fallible<MO::Distance> {
    let margin = match &trans.input_domain.context {
        Context::RowByRow { .. } => {
            return fallible!(
                MakeTransformation,
                "c-stability approximation may only be conducted under aggregation"
            )
        }
        Context::Grouping { margin, .. } => margin,
    };

    let d_in = match margin.public_info {
        // smallest valid dataset distance is 2 in bounded-DP
        Some(MarginPub::Lengths) => 2,
        _ => 1,
    };
    trans.map(&(margin.l_0(d_in), d_in, margin.l_inf(d_in)))
}
