use opendp_derive::bootstrap;
use polars_plan::dsl::Expr;

use crate::{
    combinators::{CompositionMeasure, make_approximate},
    core::{Measure, Measurement, Metric, MetricSpace, Transformation},
    domains::{AtomDomain, Context, ExprDomain, ExprPlan, Invariant, VectorDomain, WildExprDomain},
    error::Fallible,
    measurements::{
        MakeNoise, TopKMeasure,
        expr_dp_counting_query::{DPCountShim, DPLenShim, DPNUniqueShim, DPNullCountShim},
        expr_dp_frame_len::DPFrameLenShim,
        expr_dp_mean::DPMeanShim,
        expr_dp_median::DPMedianShim,
        expr_dp_quantile::DPQuantileShim,
        expr_dp_sum::DPSumShim,
        expr_noise::NoiseExprMeasure,
    },
    measures::Approximate,
    metrics::L01InfDistance,
    polars::{get_disabled_features_message, match_shim},
    transformations::{StableExpr, traits::UnboundedMetric},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_counting_query;
#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_frame_len;
#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_mean;
#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_median;
#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_quantile;
#[cfg(feature = "contrib")]
pub(crate) mod expr_dp_sum;

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
pub(crate) mod expr_noisy_max;

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
) -> Fallible<Measurement<WildExprDomain, MI, MO, ExprPlan>>
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
    ) -> Fallible<Measurement<WildExprDomain, MI, MO, ExprPlan>>;
}

impl<MI: 'static + UnboundedMetric, MO: NoiseExprMeasure + TopKMeasure + CompositionMeasure>
    PrivateExpr<L01InfDistance<MI>, MO> for Expr
where
    Expr: StableExpr<L01InfDistance<MI>, MO::Metric>,
    (ExprDomain, MO::Metric): MetricSpace,
    // This is ugly, but necessary because the necessary trait bound spans TIA
    MO::Distribution: MakeNoise<VectorDomain<AtomDomain<u32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<u64>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i8>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i16>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<i64>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<f32>>, MO::Metric, MO>
        + MakeNoise<VectorDomain<AtomDomain<f64>>, MO::Metric, MO>,
    (VectorDomain<AtomDomain<u32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<u64>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i8>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i16>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<i64>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<f32>>, MO::Metric): MetricSpace,
    (VectorDomain<AtomDomain<f64>>, MO::Metric): MetricSpace,
{
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: L01InfDistance<MI>,
        output_measure: MO,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, MO, ExprPlan>> {
        if match_shim::<DPFrameLenShim, 1>(&self)?.is_some() {
            return expr_dp_frame_len::make_expr_dp_frame_len(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }
        macro_rules! counting_query {
            ($plugin:ident, $constructor:ident) => {
                if match_shim::<$plugin, 2>(&self)?.is_some() {
                    return expr_dp_counting_query::$constructor(
                        input_domain,
                        input_metric,
                        output_measure,
                        self,
                        global_scale,
                    );
                }
            };
        }

        counting_query!(DPLenShim, make_expr_dp_len);
        counting_query!(DPCountShim, make_expr_dp_count);
        counting_query!(DPNullCountShim, make_expr_dp_null_count);
        counting_query!(DPNUniqueShim, make_expr_dp_n_unique);

        if match_shim::<DPSumShim, 4>(&self)?.is_some() {
            return expr_dp_sum::make_expr_dp_sum(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }

        if match_shim::<DPMedianShim, 3>(&self)?.is_some() {
            return expr_dp_median::make_expr_dp_median(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }

        if match_shim::<DPQuantileShim, 4>(&self)?.is_some() {
            return expr_dp_quantile::make_expr_dp_quantile(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }

        if match_shim::<DPMeanShim, 4>(&self)?.is_some() {
            return expr_dp_mean::make_expr_dp_mean(
                input_domain,
                input_metric,
                output_measure,
                self,
                global_scale,
            );
        }

        if expr_noise::match_noise(&self)?.is_some() {
            return expr_noise::make_expr_noise(input_domain, input_metric, self, global_scale);
        }

        if expr_noisy_max::match_noisy_max(&self)?.is_some() {
            return expr_noisy_max::make_expr_noisy_max(
                input_domain,
                input_metric,
                self,
                global_scale,
            );
        }

        if expr_index_candidates::match_index_candidates(&self)?.is_some() {
            return expr_index_candidates::make_expr_index_candidates(
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

        match &self {
            #[cfg(feature = "contrib")]
            Expr::Len => {
                expr_len::make_expr_private_len(input_domain, input_metric, output_measure, self)
            }

            #[cfg(feature = "contrib")]
            Expr::Literal(_) => {
                expr_literal::make_expr_private_lit(input_domain, input_metric, self)
            }

            expr => fallible!(
                MakeMeasurement,
                "Expr is not recognized at this time: {:?}. {}If you would like to see this supported, please file an issue.",
                expr,
                get_disabled_features_message()
            ),
        }
    }
}

impl<MI: 'static + UnboundedMetric, MO: 'static + Measure>
    PrivateExpr<L01InfDistance<MI>, Approximate<MO>> for Expr
where
    Expr: PrivateExpr<L01InfDistance<MI>, MO>,
{
    fn make_private(
        self,
        input_domain: WildExprDomain,
        input_metric: L01InfDistance<MI>,
        output_measure: Approximate<MO>,
        global_scale: Option<f64>,
    ) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, Approximate<MO>, ExprPlan>> {
        make_approximate(self.make_private(
            input_domain,
            input_metric,
            output_measure.0,
            global_scale,
        )?)
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
    trans: &Transformation<WildExprDomain, L01InfDistance<MI>, ExprDomain, MO>,
) -> Fallible<MO::Distance> {
    let margin = match &trans.input_domain.context {
        Context::RowByRow { .. } => {
            return fallible!(
                MakeTransformation,
                "c-stability approximation may only be conducted under aggregation"
            );
        }
        Context::Aggregation { margin, .. } => margin,
    };

    let d_in = match margin.invariant {
        // smallest valid dataset distance is 2 in bounded-DP
        Some(Invariant::Lengths) => 2,
        _ => 1,
    };
    trans.map(&(margin.l_0(d_in), d_in, margin.l_inf(d_in)))
}
