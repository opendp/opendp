use crate::core::{Measure, MetricSpace, PrivacyMap};
use crate::domains::MarginPub;
use crate::metrics::PartitionDistance;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    domains::ExprDomain,
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;
use polars_plan::dsl::len;
use polars_plan::logical_plan::LogicalPlan;

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
/// * `output_measure` - how to measure privacy loss
/// * `expr` - count expression
pub fn make_expr_private_length<MI: 'static + UnboundedMetric, MO: 'static + Measure>(
    input_domain: ExprDomain,
    input_metric: PartitionDistance<MI>,
    output_measure: MO,
    expr: Expr,
) -> Fallible<Measurement<ExprDomain, Expr, PartitionDistance<MI>, MO>>
where
    MO::Distance: Zero,
    (ExprDomain, PartitionDistance<MI>): MetricSpace,
{
    // You'd typically expect .len() and .count() to be transformations, where they are 1-stable.
    // But this constructor isn't actually a transformation;
    //    it's a measurement for releasing insensitive partition lengths (len()).
    // The sensitivity of .count() or .len() isn't zero, because changing any one record to null will change the count.
    // This is why we don't allow the variant of this expression that only counts non-null values.
    let Expr::Len = expr else {
        return fallible!(MakeMeasurement, "Expected len() expression");
    };

    let by = input_domain.context.grouping_columns()?;
    let margin = input_domain
        .frame_domain
        .margins
        .get(&by)
        .ok_or_else(|| err!(MakeMeasurement, "Unknown margin for {:?}", by))?;

    if Some(MarginPub::Lengths) != margin.public_info {
        return fallible!(
            MakeMeasurement,
            "The length of partitions when grouped by {:?} is not public information.",
            by
        );
    }

    Measurement::new(
        input_domain,
        Function::new_fallible(
            // in most other situations, we would use `Function::new_expr`, but we need to return a Fallible here
            move |(_, expr): &(LogicalPlan, Expr)| -> Fallible<Expr> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "Expected all() as input (denoting that all columns are selected). This is because column selection is a leaf node in the expression tree."
                    );
                }
                Ok(len())
            },
        ),
        input_metric,
        output_measure,
        PrivacyMap::new(move |_| MO::Distance::zero()),
    )
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
        // This will succeed because there is a margin for "chunk_2_bool" that indicates that partition lengths are public.
        let expr_domain = lf_domain.aggregate(["chunk_2_bool"]);

        let m_lap = len().make_private(
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
        let df_exact = lf.group_by([col("chunk_2_bool")]).agg([len()]).collect()?;

        assert_eq!(
            df_actual.sort(["chunk_2_bool"], false, false)?,
            df_exact.sort(["chunk_2_bool"], false, false)?
        );
        Ok(())
    }

    #[test]
    fn test_make_count_expr_no_length() -> Fallible<()> {
        let (lf_domain, _) = get_test_data()?;
        // This will fail because there is no margin for "cycle_5_alpha" that indicates that partition lengths are public.
        let expr_domain = lf_domain.aggregate(["cycle_5_alpha"]);

        let variant = len()
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
