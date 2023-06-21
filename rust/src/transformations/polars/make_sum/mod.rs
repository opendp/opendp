use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{ExprDomain, LazyFrameDomain};
use crate::error::*;
use crate::metrics::{IntDistance, L1Distance, SymmetricDistance};
use polars::prelude::*;
use std::collections::HashMap;

/// Polars operator to compute sum of a serie in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
pub fn make_sum_expr(
    input_domain: ExprDomain,
    input_metric: SymmetricDistance,
) -> Fallible<Transformation<ExprDomain, ExprDomain, SymmetricDistance, L1Distance<f64>>> {
    // Verify that the sum of ative_column can by computed //
    let active_column = input_domain.active_column.clone().unwrap_or_default();

    // Verify active_column in dataframe
    if let Some(series_domain) = input_domain.lazy_frame_domain.column(&active_column) {
        let active_column_type: DataType = series_domain.field.dtype.clone();
        if active_column_type == DataType::Utf8 {
            return fallible!(
                FailedFunction,
                "{} is of String type, can not compute sum",
                active_column
            );
        }
    } else {
        // maybe not relevant as already done on make_col
        return fallible!(MakeTransformation, "{} is not in dataframe", active_column);
    }

    // For output domain (or do as in make_col for the case with Agg context ?)
    let series_domain = input_domain.clone().lazy_frame_domain.series_domains;
    let output_domain = ExprDomain {
        lazy_frame_domain: LazyFrameDomain {
            series_domains: series_domain,
            margins: HashMap::new(), // VERIFY: no more margin
        },
        context: input_domain.context.clone(),
        active_column: input_domain.active_column.clone(),
    };

    // For StabilityMap: Compute range //
    // Get bounds from dataframe SeriesDomain of active_column
    // ASK: Otherwise are the bounds given as argument of the transformation ?
    let bounds = (1, 4); // TODO: get it
    let size_limit = 100; // TODO: get it

    let (lower, upper) = bounds;
    let ideal_sensitivity = upper
        .inf_sub(&lower)?
        .total_max(lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = S::relaxation(size_limit, lower, upper)?;

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(
            move |(expr, lf): &(Expr, LazyFrame)| -> Fallible<(Expr, LazyFrame)> {
                Ok((expr.sum(), lf.clone()))
            },
        ),
        input_metric,
        L1Distance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            S::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)
                .inf_add(&relaxation)
        }),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::domains::{AtomDomain, SeriesDomain};

    use super::*;

    fn get_test_data() -> (ExprDomain, LazyFrame) {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])
        .unwrap()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap().lazy())
        .unwrap()
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]].unwrap().lazy())
        .unwrap();

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: Context::Select, // QUESTION: should implement for all Context ?
            active_column: "A",       // because col("") operator already called
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],)
        .unwrap()
        .lazy();

        (expr_domain, lazy_frame)
    }

    #[test]
    fn test_make_sum_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data();

        // Get resulting sum (expression result)
        let expression = make_sum(expr_domain, SymmetricDistance::default())?;
        let expr_res = expression.invoke(&(all(), lazy_frame)).unwrap_test().0;

        // Get expected sum
        let expr_exp = lazy_frame.select(pl.col("A").sum());

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }

    #[test]
    fn test_make_sum_expr_domain() -> Fallible<()> {
        let (expr_domain, _) = get_test_data();

        // Get resulting ExpressionDomain
        let expression = make_sum(expr_domain, SymmetricDistance::default())?;
        let expr_domain_res = expression.output_domain.clone();

        // Get expected ExpressionDomain (output domain = input domain, no?)
        let expr_domain_exp = expr_domain.context.clone();

        assert_eq!(expr_domain_res, expr_domain_exp);

        Ok(())
    }

    #[test]
    fn test_make_sum_expr_no_active_column() -> Fallible<()> {
        // copied from make_col
        let (expr_domain, _) = get_test_data();

        // Get resulting ExpressionDomain
        let error_res = make_sum(expr_domain, SymmetricDistance::default())
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
