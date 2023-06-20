use polars::prelude::*;
use std::collections::BTreeSet;

use crate::core::{Function, MetricSpace, StabilityMap, Transformation};
use crate::domains::{Context, DatasetMetric, ExprDomain, LazyFrameDomain};
use crate::error::*;
use crate::metrics::SymmetricDistance;

pub fn make_col<M>(
    input_domain: ExprDomain,
    input_metric: M,
    col_name: &str,
) -> Fallible<Transformation<ExprDomain, ExprDomain, M, M>>
where
    M: DatasetMetric,
    (ExprDomain, M): MetricSpace,
{
    if input_domain.lazy_frame_domain.column(col_name).is_none() {
        return fallible!(MakeTransformation, "{} is not in dataframe", col_name);
    }

    let context_columns = match input_domain.context {
        Context::Agg { ref columns } => columns.clone(),
        _ => vec![],
    };

    let mut columns_to_keep = BTreeSet::from_iter(context_columns);
    columns_to_keep.insert(String::from(col_name));

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

    let output_domain = ExprDomain {
        lazy_frame_domain: LazyFrameDomain {
            series_domains,
            margins,
        },
        context: input_domain.context.clone(),
        active_column: Some(col_name.to_string()),
    };

    let column_name = col_name.to_string();

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(
            move |(expr, lazyframe): &(Expr, LazyFrame)| -> Fallible<(Expr, LazyFrame)> {
                if expr != &Expr::Wildcard {
                    return fallible!(
                        FailedFunction,
                        "make_col has to be the first expression in the expression chain"
                    );
                }
                Ok((col(&*column_name), lazyframe.clone()))
            },
        ),
        input_metric.clone(),
        input_metric.clone(),
        StabilityMap::new_from_constant(1),
    )
}

#[cfg(test)]
mod test_make_col {
    use crate::domains::{AtomDomain, SeriesDomain};

    use super::*;

    fn get_test_data() -> (ExprDomain, LazyFrame) {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])
        .unwrap_test()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap().lazy())
        .unwrap_test()
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]].unwrap().lazy())
        .unwrap_test()
        .with_counts(df!["C" => [8, 9, 10], "count" => [1, 1, 1]].unwrap().lazy())
        .unwrap_test();

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: Context::Agg {
                columns: vec![String::from("A")],
            },
            active_column: None,
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)
        .unwrap_test()
        .lazy();

        (expr_domain, lazy_frame)
    }

    #[test]
    fn test_make_col_expr() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data();
        let selected_col = "B";
        let transformation =
            make_col(expr_domain, SymmetricDistance::default(), selected_col).unwrap_test();

        let expr_res = transformation.invoke(&(all(), lazy_frame)).unwrap_test().0;
        let expr_exp = col(selected_col);

        assert_eq!(expr_res, expr_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_domain() -> Fallible<()> {
        let (expr_domain, _) = get_test_data();
        let expr_domain_context_exp = expr_domain.context.clone();

        let selected_col = "B";
        let transformation =
            make_col(expr_domain, SymmetricDistance::default(), selected_col).unwrap_test();

        let expr_domain_res = transformation.output_domain.clone();

        let lf_domain_exp = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
        ])
        .unwrap_test()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap_test().lazy())
        .unwrap_test()
        .with_counts(
            df!["B" => [1.0, 2.0], "count" => [2, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test();

        let expr_domain_exp = ExprDomain {
            lazy_frame_domain: lf_domain_exp,
            context: expr_domain_context_exp,
            active_column: Some(selected_col.to_string()),
        };

        assert_eq!(expr_domain_res, expr_domain_exp);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_no_wildcard() -> Fallible<()> {
        let (expr_domain, lazy_frame) = get_test_data();
        let selected_col = "B";

        let transformation =
            make_col(expr_domain, SymmetricDistance::default(), selected_col).unwrap_test();
        let error_res = transformation
            .invoke(&(col(selected_col), lazy_frame))
            .map(|v| v.0)
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::FailedFunction;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }

    #[test]
    fn test_make_col_expr_wrong_col() -> Fallible<()> {
        let (expr_domain, _) = get_test_data();
        let selected_col = "D";

        let error_res = make_col(expr_domain, SymmetricDistance::default(), selected_col)
            .map(|v| v.input_domain.clone())
            .unwrap_err()
            .variant;
        let expected_error_kind = ErrorVariant::MakeTransformation;

        assert_eq!(error_res, expected_error_kind);

        Ok(())
    }
}
