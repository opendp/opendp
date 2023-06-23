use polars::prelude::*;

use crate::core::{Function, StabilityMap, Transformation};
use crate::domains::{Context, ExprDomain, LazyFrameDomain};
use crate::error::*;
use crate::metrics::{IntDistance, SymmetricDistance};

/// Make a Transformation that applies list of transformations to a Lazy Frame.
///
pub fn make_select_trans(
    input_domain: LazyFrameDomain,
    input_metric: SymmetricDistance,
    transformations: Vec<
        Transformation<ExprDomain, ExprDomain, SymmetricDistance, SymmetricDistance>,
    >,
) -> Fallible<Transformation<LazyFrameDomain, LazyFrameDomain, SymmetricDistance, SymmetricDistance>> where
{
    let fist_transformation = transformations
        .first()
        .ok_or_else(|| err!(MakeTransformation, "transformation list cannot be empty"))?;

    let output_domain = fist_transformation.output_domain.lazy_frame_domain.clone();

    let output_metric = fist_transformation.output_metric.clone();

    if transformations
        .iter()
        .any(|t| t.input_domain.lazy_frame_domain != input_domain)
    {
        return fallible!(MakeTransformation, "input domains do not match");
    }

    if transformations
        .iter()
        .any(|t| t.output_domain.context != Context::Select)
    {
        return fallible!(
            MakeTransformation,
            "transformation is not in select context"
        );
    }

    if transformations
        .iter()
        .any(|t| t.output_domain.lazy_frame_domain != output_domain)
    {
        return fallible!(
            MakeTransformation,
            "transformations' output domains do not match"
        );
    }

    let stability_maps: Vec<_> = transformations
        .iter()
        .map(|t| &t.stability_map)
        .cloned()
        .collect();

    let stability_map = StabilityMap::new_fallible(move |d_in| {
        let distances = stability_maps
            .iter()
            .map(|map| map.eval(d_in))
            .collect::<Fallible<Vec<IntDistance>>>()?;

        Ok(*distances.iter().max().expect("distances cannot be empty"))
    });

    Transformation::new(
        input_domain.clone(),
        output_domain,
        Function::new_fallible(move |lazy_frame: &LazyFrame| -> Fallible<LazyFrame> {
            let trans_outputs = transformations
                .iter()
                .map(|t| t.invoke(&(lazy_frame.clone(), all())))
                .collect::<Fallible<Vec<(LazyFrame, Expr)>>>()?;

            let exprs: Vec<Expr> = trans_outputs.iter().map(|(_, expr)| expr.clone()).collect();

            Ok(lazy_frame.clone().select(&exprs))
        }),
        input_metric.clone(),
        output_metric,
        stability_map,
    )
}

#[cfg(test)]
mod test_make_select_trans {
    use polars::prelude::*;

    use crate::domains::{AtomDomain, ExprDomain, LazyFrameDomain, SeriesDomain};
    use crate::transformations::make_col;
    use crate::transformations::polars::test::get_test_data;

    use super::*;

    fn get_3_row_test_data() -> (LazyFrameDomain, LazyFrame, ExprDomain) {
        let lf_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])
        .unwrap_test()
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]].unwrap_test().lazy())
        .unwrap_test()
        .with_counts(
            df!["B" => [1.0, 2.0], "count" => [2, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test()
        .with_counts(
            df!["C" => [8, 9, 10], "count" => [1, 1, 1]]
                .unwrap_test()
                .lazy(),
        )
        .unwrap_test();

        let lazy_frame = df!(
            "A" => &[1, 2, 2,],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)
        .unwrap_test()
        .lazy();

        let expr_domain = ExprDomain {
            lazy_frame_domain: lf_domain.clone(),
            context: Context::Select,
            active_column: None,
        };

        (lf_domain.clone(), lazy_frame, expr_domain)
    }

    fn get_2_row_test_data() -> (LazyFrameDomain, LazyFrame, ExprDomain) {
        let lf_domain = LazyFrameDomain::new(vec![
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

        let lazy_frame = df!(
            "A" => &[1, 2, 2,],
            "B" => &[1.0, 1.0, 2.0],)
        .unwrap_test()
        .lazy();

        let expr_domain = ExprDomain {
            lazy_frame_domain: lf_domain.clone(),
            context: Context::Select,
            active_column: None,
        };

        (lf_domain.clone(), lazy_frame, expr_domain)
    }

    #[test]
    fn test_make_select_trans_lazy_frame() -> Fallible<()> {
        let (lf_domain, lazy_frame, expr_domain) = get_3_row_test_data();
        let select_trans = make_select_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![make_col(expr_domain, SymmetricDistance::default(), "B").unwrap_test()],
        );

        let lf_res = select_trans
            .unwrap_test()
            .invoke(&lazy_frame)
            .unwrap_test()
            .collect()
            .unwrap_test();
        let lf_exp = df!(
            "B" => &[1.0, 1.0, 2.0],)
        .unwrap_test();

        assert!(lf_exp.frame_equal(&lf_res));

        Ok(())
    }

    #[test]
    fn test_make_select_trans_output_domain() -> Fallible<()> {
        let (lf_domain, _, expr_domain) = get_3_row_test_data();
        let select_trans = make_select_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![make_col(expr_domain, SymmetricDistance::default(), "B").unwrap_test()],
        )
        .unwrap_test();

        let output_domain_res = &select_trans.output_domain;

        let output_domain_exp =
            LazyFrameDomain::new(vec![SeriesDomain::new("B", AtomDomain::<f64>::default())])
                .unwrap_test()
                .with_counts(
                    df!["B" => [1.0, 2.0], "count" => [2, 1]]
                        .unwrap_test()
                        .lazy(),
                )
                .unwrap_test();

        assert_eq!(output_domain_res, &output_domain_exp);

        Ok(())
    }

    #[test]
    fn test_make_select_trans_wrong_input_domain() -> Fallible<()> {
        let (lf_domain, _, expr_domain) = get_3_row_test_data();
        let (_, _, expr_domain_2_row) = get_2_row_test_data();
        let error_msg_res = make_select_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![
                make_col(expr_domain, SymmetricDistance::default(), "B").unwrap_test(),
                make_col(expr_domain_2_row, SymmetricDistance::default(), "A").unwrap_test(),
            ],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .message;
        let error_msg_exp = Some("input domains do not match".to_string());

        assert_eq!(error_msg_res, error_msg_exp);

        Ok(())
    }

    #[test]
    fn test_make_select_trans_wrong_context() -> Fallible<()> {
        let (lf_domain, _, mut expr_domain) = get_3_row_test_data();
        expr_domain.context = Context::Filter;

        let error_msg_res = make_select_trans(
            lf_domain,
            SymmetricDistance::default(),
            vec![
                make_col(expr_domain.clone(), SymmetricDistance::default(), "B").unwrap_test(),
                make_col(expr_domain.clone(), SymmetricDistance::default(), "A").unwrap_test(),
            ],
        )
        .map(|v| v.input_domain.clone())
        .unwrap_err()
        .message;
        let error_msg_exp = Some("transformation is not in select context".to_string());

        assert_eq!(error_msg_res, error_msg_exp);

        Ok(())
    }

    #[test]
    fn test_make_select_trans_empty_list() -> Fallible<()> {
        let (lf_domain, _, _) = get_3_row_test_data();
        let error_msg_res = make_select_trans(lf_domain, SymmetricDistance::default(), vec![])
            .map(|v| v.input_domain.clone())
            .unwrap_err()
            .message;
        let error_msg_exp = Some("transformation list cannot be empty".to_string());

        assert_eq!(error_msg_res, error_msg_exp);

        Ok(())
    }
}
