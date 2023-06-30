use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, ExprMetric};
use crate::error::*;
use crate::metrics::{InsertDeleteDistance, IntDistance, L1Distance};
use crate::traits::{InfAdd, InfCast, InfMul, InfSub};
use crate::transformations::{Sequential, SumRelaxation, CanFloatSumOverflow};
use polars::prelude::*;
use std::collections::BTreeSet;

/// Polars operator to compute sum of a series in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
pub fn make_sum_expr<MI, C: Context>(
    input_domain: ExprDomain<C>,
    input_metric: MI,
) -> Fallible<Transformation<ExprDomain<C>, ExprDomain<C>, MI, L1Distance<f64>>>
where
    MI: ExprMetric<C, InnerMetric = InsertDeleteDistance, Distance = IntDistance>,
    (ExprDomain<C>, MI): MetricSpace,
    (ExprDomain<C>, L1Distance<f64>): MetricSpace,
{
    // output domain
    let mut output_domain = input_domain.clone();
    (output_domain.lazy_frame_domain)
        .try_column_mut(input_domain.active_column()?)?
        .drop_bounds()?;

    // stability map
    let bounds = input_domain
        .active_series()?
        .atom_domain::<f64>()?
        .get_closed_bounds()?;
    let (upper, lower) = bounds;

    let ideal_sensitivity = upper.inf_sub(&lower)?;

    let margin = input_domain
        .lazy_frame_domain
        .margins
        .get(&BTreeSet::from_iter(
            input_domain.context.grouping_columns(),
        ))
        .ok_or_else(|| err!(MakeTransformation, "failed to find margin"))?;
    let max_size = margin.get_max_size()? as usize;

    if Sequential::<f64>::float_sum_can_overflow(max_size, (lower, upper))? {
        return fallible!(
            MakeTransformation,
            "potential for overflow when computing function"
        );
    }

    let relaxation = Sequential::<f64>::relaxation(max_size, lower, upper)?;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new(
            move |(frame, expr): &(C::Value, Expr)| -> (C::Value, Expr) {
                (frame.clone(), expr.clone().sum())
            },
        ),
        input_metric,
        L1Distance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            // TODO: in the future, incorporate bounds on the number of partitions from the metric
            let max_changed_partitions = if C::GROUPBY {
                f64::inf_cast(*d_in)?
            } else {
                1.0
            };
            f64::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&max_changed_partitions.inf_mul(&relaxation)?)
        }),
    )
}

#[cfg(test)]
mod test_make_sum_expr {
    use crate::{
        domains::{AtomDomain, LazyFrameDomain, LazyGroupByContext, SeriesDomain},
        metrics::Lp,
    };

    use super::*;

    fn get_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame.groupby([col("A")])))
    }

    #[test]
    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr(expr_domain, Lp(InsertDeleteDistance))?;
        let expr_res = trans.invoke(&(lazy_frame.clone(), col("B")))?.1;
        let frame_actual = lazy_frame
            .clone()
            .agg([expr_res])
            .sort("A", Default::default())
            .collect()?;

        // Get expected sum
        let frame_expected = lazy_frame
            .agg([col("B").sum()])
            .sort("A", Default::default())
            .collect()?;

        assert_eq!(frame_actual, frame_expected);

        Ok(())
    }
}
