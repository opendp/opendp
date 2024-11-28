use crate::core::{Measure, PrivacyMap};
use crate::domains::{MarginPub, WildExprDomain};
use crate::metrics::PartitionDistance;
use crate::transformations::traits::UnboundedMetric;
use crate::{
    core::{Function, Measurement},
    error::Fallible,
};

use num::Zero;
use polars::lazy::dsl::Expr;
use polars_plan::dsl::len;

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
    input_metric: PartitionDistance<MI>,
    output_measure: MO,
    expr: Expr,
) -> Fallible<Measurement<WildExprDomain, Expr, PartitionDistance<MI>, MO>>
where
    MO::Distance: Zero,
{
    let Expr::Len = expr else {
        return fallible!(MakeMeasurement, "Expected len() expression");
    };

    let (by, margin) = input_domain.context.grouping("len")?;

    if Some(MarginPub::Lengths) != margin.public_info {
        return fallible!(
            MakeMeasurement,
            "The length of partitions when grouped by {:?} is not public information. You may have forgotten to add noise to your query.",
            by
        );
    }

    Measurement::new(
        input_domain,
        Function::new(|_| len()),
        input_metric,
        output_measure,
        PrivacyMap::new(move |_| MO::Distance::zero()),
    )
}
