use crate::core::{ExprFunction, Measure, MetricSpace, PrivacyMap};
use crate::domains::MarginPub;
use crate::metrics::PartitionDistance;
use crate::transformations::traits::UnboundedMetric;
use crate::transformations::StableExpr;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;
use polars_plan::dsl::AggExpr;

/// Make a measurement that computes the count exactly under bounded-DP
///
/// | input_metric                              |
/// | ----------------------------------------- |
/// | `PartitionDistance<SymmetricDistance>`    |
/// | `PartitionDistance<InsertDeleteDistance>` |
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - valid selections shown in table above
/// * `expr` - count expression
pub fn make_expr_private_count<MI: 'static + UnboundedMetric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    expr: Expr,
) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<MI>, MO>>
where
    MO::Distance: Zero,
    Expr: StableExpr<PartitionDistance<MI>, PartitionDistance<MI>>,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    let Expr::Agg(AggExpr::Count(expr, include_nulls)) = expr else {
        return fallible!(MakeMeasurement, "Expected Count expression");
    };

    if !include_nulls {
        return fallible!(
            MakeMeasurement,
            "Public margin sizes assume nulls are included. Try using `len` instead."
        );
    }

    let t_prior = expr
        .clone()
        .make_stable(input_domain.clone(), input_metric.clone())?;

    let (middle_domain, middle_metric) = t_prior.output_space();

    let by = middle_domain.context.grouping_columns()?;
    let margin = middle_domain
        .frame_domain
        .margins
        .get(&by)
        .ok_or_else(|| err!(MakeMeasurement, "Unknown margin for {:?}", by))?;

    if Some(MarginPub::Lengths) != margin.public_info {
        return fallible!(
            MakeMeasurement,
            "Len is only private if size(s) of {:?} are public",
            by
        );
    }

    t_prior
        >> Measurement::new(
            middle_domain,
            Function::new_expr(|input_expr| input_expr.len()),
            middle_metric,
            MO::default(),
            PrivacyMap::new(move |_| MO::Distance::zero()),
        )?
}

#[cfg(test)]
mod test_make_expr_count_private {
    use super::*;
    use polars::prelude::*;

    use crate::{
        error::ErrorVariant,
        measurements::PrivateExpr,
        measures::MaxDivergence,
        metrics::{PartitionDistance, SymmetricDistance},
        transformations::test_helper::get_test_data,
    };

    #[test]
    fn test_make_count_expr_grouped() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["chunk_2_bool"]);

        let m_lap = col("const_1f64").len().make_private(
            expr_domain,
            PartitionDistance(SymmetricDistance),
            MaxDivergence::default(),
            None,
        )?;

        let meas_res = m_lap.invoke(&(lf.logical_plan.clone(), all()))?;

        let df_actual = lf
            .clone()
            .group_by([col("chunk_2_bool")])
            .agg([meas_res])
            .collect()?;
        let df_exact = lf
            .group_by([col("chunk_2_bool")])
            .agg([col("const_1f64").len()])
            .collect()?;

        assert_eq!(
            df_actual.sort(["chunk_2_bool"], false, false)?,
            df_exact.sort(["chunk_2_bool"], false, false)?
        );
        Ok(())
    }

    #[test]
    fn test_make_count_expr_no_length() -> Fallible<()> {
        let (lf_domain, _) = get_test_data()?;
        let expr_domain = lf_domain.aggregate(["cycle_5_alpha"]);

        let variant = col("const_1f64")
            .len()
            .make_private(
                expr_domain,
                PartitionDistance(SymmetricDistance),
                MaxDivergence::default(),
                None,
            )
            .map(|_| ())
            .unwrap_err()
            .variant;

        assert_eq!(variant, ErrorVariant::MakeMeasurement);
        Ok(())
    }
}
