use polars::prelude::*;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{ExprDomain, OuterMetric};
use crate::error::{ErrorVariant::MakeTransformation, *};
use crate::transformations::DatasetMetric;

/// Make a Transformation that returns a `col(column_name)` expression for a Lazy Frame.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
/// * `expr` - A column expression.
pub fn make_expr_col<M: OuterMetric>(
    input_domain: ExprDomain,
    input_metric: M,
    expr: Expr,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M::InnerMetric: DatasetMetric,
    M::Distance: Clone,
    (ExprDomain, M): MetricSpace,
{
    let Expr::Column(col_name) = expr else {
        return fallible!(MakeTransformation, "Expected col() expression");
    };
    let col_name = col_name.to_string();

    let mut output_domain = input_domain.clone();
    output_domain
        .frame_domain
        .series_domains
        .retain(|v| v.field.name == col_name);

    output_domain
        .check_one_column()
        .with_variant(MakeTransformation)?;

    Transformation::new(
        input_domain,
        output_domain,
        Function::new_fallible(
            // in most other situations, we would use `Function::new_expr`, but we need to return a Fallible here
            move |(plan, expr): &(LogicalPlan, Expr)| -> Fallible<(LogicalPlan, Expr)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "Expected all() as input (denoting that all columns are selected). This is because column selection is a leaf node in the expression tree."
                    );
                }
                Ok((plan.clone(), col(&*col_name)))
            },
        ),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

#[cfg(test)]
mod test {
    use crate::metrics::SymmetricDistance;
    use crate::transformations::{test::get_test_data, StableExpr};

    use super::*;

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.row_by_row();
        let expected = col("const_1f64");
        let t_col = expected
            .clone()
            .make_stable(expr_domain.clone(), SymmetricDistance)?;
        let actual = t_col.invoke(&(lf.logical_plan, all()))?.1;

        assert_eq!(actual, expected);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_no_wildcard() -> Fallible<()> {
        let (lf_domain, lf) = get_test_data()?;
        let expr_domain = lf_domain.row_by_row();

        let t_col = col("const_1f64").make_stable(expr_domain.clone(), SymmetricDistance)?;
        let error_res = t_col
            .invoke(&(lf.logical_plan, col("not wildcard")))
            .map(|v| v.1)
            .unwrap_err()
            .variant;
        assert_eq!(error_res, ErrorVariant::FailedFunction);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_wrong_col() -> Fallible<()> {
        let (lf_domain, _) = get_test_data()?;
        let expr_domain = lf_domain.row_by_row();

        let variant = col("nonexistent")
            .make_stable(expr_domain, SymmetricDistance)
            .map(|_: Transformation<_, _, _, SymmetricDistance>| ())
            .unwrap_err()
            .variant;

        assert_eq!(variant, ErrorVariant::MakeTransformation);
        Ok(())
    }
}
