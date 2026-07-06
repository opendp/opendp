use crate::core::{Measure, PrivacyMap};
use crate::domains::{ExprPlan, Invariant, WildExprDomain};
use crate::metrics::L01InfDistance;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;
use polars::prelude::{DataType, len};
use polars_plan::plans::typed_lit;

#[cfg(test)]
mod test;

/// Make a measurement that computes the count exactly under bounded-DP
///
/// In the setting of bounded-DP, the data set length is public information.
/// Therefore a simple count of the number of records in the data set satisfies 0-DP.
///
/// A similar logic applies to the setting where partition sizes are public information.
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
pub fn make_expr_private_len<MI: 'static + UnboundedMetric, MO: 'static + Measure>(
    input_domain: WildExprDomain,
    input_metric: L01InfDistance<MI>,
    output_measure: MO,
    expr: Expr,
) -> Fallible<Measurement<WildExprDomain, L01InfDistance<MI>, MO, ExprPlan>>
where
    MO::Distance: Zero,
{
    let margin = input_domain.context.aggregation("len")?;

    if Some(Invariant::Lengths) != margin.invariant {
        return fallible!(
            MakeMeasurement,
            "The length of partitions when grouped by {:?} is not public information. You may have forgotten to add noise to your query.",
            margin.by
        );
    }

    match expr {
        Expr::Len => Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::from_expr(len()).fill_with(typed_lit(0u32)),
            PrivacyMap::new(move |_| MO::Distance::zero()),
        ),

        Expr::Cast { expr, .. } => {
            if matches!(expr.as_ref(), Expr::Len) {
                Measurement::new(
                    input_domain,
                    input_metric,
                    output_measure,
                    Function::from_expr(len().cast(DataType::Int64)).fill_with(typed_lit(0i64)),
                    PrivacyMap::new(move |_| MO::Distance::zero()),
                )
            } else {
                return fallible!(
                    MakeMeasurement,
                    "Expected len().cast() and got an unsupported expression"
                );
            }
        }
        _ => {
            return fallible!(MakeMeasurement, "Expected len() or len().cast() expression");
        }
    }
}
