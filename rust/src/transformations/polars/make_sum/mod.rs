use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{AtomDomain, Context, ExprDomain, LazyGroupByContext, GroupingColumns};
use crate::error::*;
use crate::metrics::{InsertDeleteDistance, IntDistance, L1Distance, L1};
use crate::traits::{AlertingAbs, InfAdd, InfCast, InfMul, InfSub, TotalOrd};
use crate::transformations::{Sequential, SumRelaxation};
use polars::prelude::*;
use std::collections::BTreeSet;

/// Polars operator to compute sum of a serie in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
pub fn make_sum_expr(
    input_domain: ExprDomain<LazyGroupByContext>,
    input_metric: L1<InsertDeleteDistance>,
) -> Fallible<Transformation<ExprDomain<LazyGroupByContext>, ExprDomain<LazyGroupByContext>, L1<InsertDeleteDistance>, L1Distance<f64>>> {
    // Verify that the sum of ative_column can by computed //
    let active_column = input_domain.active_column.clone()
        .ok_or_else(|| err!(MakeTransformation, "No active column"))?;

    // Output domain -- TODO: this could probably be written cleaner
    let mut output_domain = input_domain.clone();
    let active_column_index = output_domain.lazy_frame_domain.column_index(&active_column)
        .ok_or_else(|| err!(MakeTransformation, "could not find index"))?;
    let mut series_domains = output_domain.lazy_frame_domain.series_domains;
    series_domains[active_column_index] = series_domains[active_column_index].clone().drop_bounds()?;
    output_domain.lazy_frame_domain.series_domains = series_domains;
    
    // For StabilityMap
    let bounds = input_domain
        .lazy_frame_domain
        .try_column(&active_column)?
        .element_domain
        .as_any()
        .downcast_ref::<AtomDomain<f64>>()
        .ok_or_else(|| err!(MakeTransformation, "Failed to downcast: {}", &active_column))?
        .get_closed_bounds()
        .ok_or_else(|| err!(MakeTransformation, "Bounds must be set"))?;
    let (lower, upper) = bounds;

    let ideal_sensitivity = upper
        .inf_sub(&lower)?
        .total_max(lower.alerting_abs()?.total_max(upper)?)?;

    let grouping_columns = input_domain.context.grouping_columns();
    
    let margin = input_domain
        .lazy_frame_domain
        .margins
        .get(&BTreeSet::from_iter(grouping_columns.clone()))
        .ok_or_else(|| err!(MakeTransformation, "Failed to find margin"))?;

    let max_size = margin.get_max_size()?;
    let relaxation = Sequential::<f64>::relaxation(max_size as usize, lower, upper)?;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(
            move |(expr, lf): &(Expr, LazyGroupBy)| -> Fallible<(Expr, LazyGroupBy)> {
                Ok((expr.clone().sum(), lf.clone()))
            },
        ),
        input_metric,
        L1Distance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            f64::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
                .inf_add(&f64::inf_cast(*d_in)?.inf_mul(&relaxation)?)
        }),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::domains::{AtomDomain, SeriesDomain, LazyFrameDomain};

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
            "B" => &[1.0, 1.0, 2.0],)?.lazy();

        Ok((expr_domain, lazy_frame.groupby([col("A")])))
    }

    #[test]
    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr(expr_domain, InsertDeleteDistance::default())?;
        let expr_res = trans.invoke(&(col("B"), lazy_frame.clone()))?.0;
        let frame_actual = lazy_frame.clone().groupby([col("A")]).agg([expr_res]).collect()?;

        // Get expected sum
        let frame_expected = lazy_frame.groupby([col("A")]).agg([col("B").sum()]).collect()?;

        assert_eq!(frame_actual, frame_expected);

        Ok(())
    }

    #[test]
    fn test_fail_if_no_bounds() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;

        // Get resulting sum (expression result)
        let trans = make_sum_expr(expr_domain, InsertDeleteDistance::default())?;
        let expr_res = trans.invoke(&(col("B"), lazy_frame.clone()))?.0;
        let frame_actual = lazy_frame.clone().groupby([col("A")]).agg([expr_res]).collect()?;

        // Get expected sum
        let frame_expected = lazy_frame.groupby([col("A")]).agg([col("B").sum()]).collect()?;

        assert_eq!(frame_actual, frame_expected);

        Ok(())
    }

    #[test]
    fn test_make_sum_expr_no_active_column() -> Fallible<()> {
        // copied from make_col
        let (expr_domain, _) = get_test_data()?;

        // Get resulting ExpressionDomain
        let error_res = make_sum_expr(expr_domain, InsertDeleteDistance::default())
            .map(|v| v.input_domain.clone())
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::MakeTransformation;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }

    #[test]
    fn test_make_sum_expr_no_counts() -> Fallible<()> {
        // TODO: does not have counts (don't know size)

        Ok(())
    }

    #[test]
    fn test_make_sum_expr_no_bounds() -> Fallible<()> {
        // TODO: does not have bounds (don't know snesibility)

        Ok(())
    }
}
