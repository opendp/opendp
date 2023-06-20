use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{DatasetMetric, LazyFrameDomain};
use crate::metrics::{IntDistance, L1Distance, SymmetricDistance};
use polars::prelude::*;

/// Polars operator to compute sum of a serie in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
pub fn make_sum_expr(
    input_domain: ExprDomain,
    input_metric: SymmetricDistance,
) -> Fallible<Transformation<ExprDomain, ExprDomain, SymmetricDistance, L1Distance<f64>>> {
    // Output domain has the same series as input domain but QUESTION maybe lose margins ?
    let output_domain = input_domain.clone();

    // VERIFY: active_column is the column that was selected with make_col ?
    let active_column = input_domain.clone().active_column;

    // Verify active_column in dataframe (maybe not relevant as already done on make_col)
    if input_domain
        .lazy_frame_domain
        .column(active_column.clone())
        .is_none()
    {
        return fallible!(MakeTransformation, "{} is not in dataframe", col_name);
    }

    // For StabilityMap: Compute ideal_sensitivity and relaxation //
    // Verify active_column has margin with counts (to compute size for stability map)
    let active_column_margin = input_domain
        .clone()
        .lazy_frame_domain
        .margins
        .into_iter()
        .filter(|(s, _)| s.contains(active_column.clone()))
        .collect();
    let lazy_frame_size = active_column_margin.sum(); // get size of df

    // TODO: get bounds from dataframe SeriesDomain of active_column
    // ASK: Otherwise are the bounds given as argument of the transformation ?
    let bounds = input_domain
        .lazy_frame_domain
        .col(active_column.clone())
        .bounds();

    // Compute ideal sensitivity
    let lower = bounds.clone().lower();
    let upper = bounds.clone().upper();
    let ideal_sensitivity = upper.inf_sub(&lower)?;

    // Compute sensitivity correction for bit approximation
    let mantissa_bits = T::exact_int_cast(T::MANTISSA_BITS)?;
    let _2 = T::exact_int_cast(2)?;

    // Formula is: n^2 / 2^(k - 1) max(|L|, U)
    let error = lazy_frame_size
        .inf_mul(&lazy_frame_size)?
        .inf_div(&_2.inf_pow(&mantissa_bits)?)?
        .inf_mul(&lower.alerting_abs()?.total_max(upper)?)?;
    let relaxation = error.inf_add(&error)?;

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
            T::inf_cast(d_in / 2)?
                .inf_mul(&ideal_sensitivity)?
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
