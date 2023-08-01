use opendp_derive::bootstrap;
use polars::prelude::*;
use std::collections::BTreeSet;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, ExprMetric, LazyFrameDomain};
use crate::error::*;

#[bootstrap(ffi = false)]
/// Make a Transformation that returns a `col(column_name)` expression for a Lazy Frame.
///
/// # Arguments
/// * `input_domain` - Domain of the expression to be applied.
/// * `input_metric` - The metric under which neighboring LazyFrames are compared.
///
/// # Generics
/// * `M` - Dataset Metric type.
/// * `C` - Context in which expression is applied.
///
pub fn make_col<M, C: Context>(
    input_domain: ExprDomain<C>,
    input_metric: M,
    col_name: String,
) -> Fallible<Transformation<ExprDomain<C>, ExprDomain<C>, M, M>>
where
    M: ExprMetric<C>,
    M::Distance: Clone + 'static,
    (ExprDomain<C>, M): MetricSpace,
{
    if input_domain.active_column.is_some() {
        return fallible!(
            MakeTransformation,
            "Active column has to be set to none in make_col constructor"
        );
    }

    input_domain
        .lazy_frame_domain
        .try_column(col_name.as_str())?;

    let context_columns = input_domain.context.grouping_columns();

    let mut columns_to_keep = BTreeSet::from_iter(context_columns);
    columns_to_keep.insert(col_name.clone());

    let series_domains = input_domain
        .clone()
        .lazy_frame_domain
        .series_domains
        .into_iter()
        .filter(|s| columns_to_keep.contains(&s.field.name.to_string()))
        .collect();

    let margins = input_domain
        .clone()
        .lazy_frame_domain
        .margins
        .into_iter()
        .filter(|(s, _)| s.is_subset(&columns_to_keep))
        .collect();

    let output_domain = ExprDomain::new(
        LazyFrameDomain {
            series_domains,
            margins,
        },
        input_domain.context.clone(),
        Some(col_name.clone()),
        true,
    );

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(
            move |(frame, expr): &(C::Value, Expr)| -> Fallible<(C::Value, Expr)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "make_col has to be the first expression in the expression chain"
                    );
                }
                Ok((frame.clone(), col(&*col_name)))
            },
        ),
        input_metric.clone(),
        input_metric,
        StabilityMap::new(Clone::clone),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::domains::{AtomDomain, LazyGroupByContext, SeriesDomain};
    use crate::metrics::{Lp, SymmetricDistance};
    use crate::transformations::polars_test::get_test_data;

    use super::*;

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let selected_col = "B";
        let transformation = make_col(expr_domain, SymmetricDistance, selected_col.to_string())?;

        let expr_res = transformation.invoke(&(lazy_frame, all()))?.1;
        let expr_exp = col(selected_col);

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_domain() -> Fallible<()> {
        let (expr_domain, _) = get_test_data()?;

        let context = LazyGroupByContext {
            columns: vec![String::from("A")],
        };
        let expr_domain = ExprDomain::new(
            expr_domain.lazy_frame_domain,
            context.clone(),
            expr_domain.active_column,
            true,
        );
        let selected_col = "B";
        let transformation =
            make_col(expr_domain, Lp(SymmetricDistance), selected_col.to_string())?;

        let expr_domain_res = transformation.output_domain.clone();

        let lf_domain_exp = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?;

        let expr_domain_exp =
            ExprDomain::new(lf_domain_exp, context, Some(selected_col.to_string()), true);

        assert_eq!(expr_domain_res, expr_domain_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_no_wildcard() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data()?;
        let selected_col = "B";

        let transformation = make_col(expr_domain, SymmetricDistance, selected_col.to_string())?;
        let error_res = transformation
            .invoke(&(lazy_frame, col(selected_col)))
            .map(|v| v.1)
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::FailedFunction;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_wrong_col() -> Fallible<()> {
        let (expr_domain, _) = get_test_data()?;
        let selected_col = "D";

        let error_res = make_col(expr_domain, SymmetricDistance, selected_col.to_string())
            .map(|v| v.input_domain.clone())
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::FailedFunction;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }
}
