use polars::prelude::*;
use std::ops::{Mul, Sub};

use crate::{
    domains::{ExprDomain, LazyGroupByContext}, 
    metrics::{SymmetricDistance, LInfDiffDistance, IntDistance}, 
    core::{Transformation, Function, StabilityMap}, 
    error::Fallible,

};

/// Polars operator to compute quantile of a serie in a LazyFrame
///
/// # Arguments
/// * `input_domain` - ExprDomain
/// * `input_metric` - The metric space under which neighboring LazyFrames are compared
/// * `alpha` - a value in [0, 1]. Choose 0.5 for median
pub fn make_quantile_expr(
    input_domain: ExprDomain<LazyGroupByContext>,
    input_metric: SymmetricDistance,
    alpha: f64
) -> Fallible<
    Transformation<
        ExprDomain<LazyGroupByContext>,
        ExprDomain<LazyGroupByContext>,
        SymmetricDistance,
        LInfDiffDistance<f64>,
    >,
> {

    Transformation::new(
        input_domain.clone(),
        input_domain.clone(),
        Function::new_fallible(
            move |(frame, expr): &(LazyGroupBy, Expr)| -> Fallible<(LazyGroupBy, Expr)> {
                Ok((frame.clone(), make_score_elts_expr(expr.clone(), alpha))) // add exp mechanism
            },
        ),
        input_metric,
        LInfDiffDistance::default(),
        StabilityMap::new_fallible(move |d_in: &IntDistance| {
            unimplemented!();
        }),
    )
}

fn make_score_elts_expr(expr: Expr, alpha: f64) -> Expr {
    expr.sort(false)
        .slice(lit(1), lit(NULL)) // but it removes the last value
        .rank(RankOptions::default(), None)
        .cast(DataType::Float64)
        .sub(count().cast(DataType::Float64).mul(lit(alpha)))
        .abs()
}

#[cfg(test)]
mod test_make_col {

    use super::*;
    use crate::domains::{
        AtomDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain,
    };
    use crate::error::Fallible;

    fn get_select_test_data() -> Fallible<(ExprDomain<LazyFrameContext>, LazyFrame)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::new_closed((1, 4))?),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 5.5))?),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyFrameContext::Select,
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 3, 4, 5],
            "B" => &[1.0, 2.0, 3.0, 4.0, 5.0],)?
        .lazy();

        Ok((expr_domain, lazy_frame))
    }

    #[test]
    fn test_make_score_elts_expr_select() -> Fallible<()> {
        let (_, lazy_frame) = get_select_test_data()?;

        let expr = col("B");
        let expr_make_score = make_score_elts_expr(expr, 0.5);

        // Get resulting scoring
        let frame_actual = lazy_frame.clone().select([expr_make_score]).collect()?;

        // Get expected scoring
        let frame_expected = df!(
            "B" => &[1.5, 0.5, 0.5, 1.5], // "B" => &[1.5, 0.5, 0.5, 1.5, 2.5], without slice
        )?;

        assert_eq!(frame_actual, frame_expected);
        Ok(())
    }

    pub fn get_groupby_test_data() -> Fallible<(ExprDomain<LazyGroupByContext>, LazyGroupBy)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<i32>::default()),
        ])?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain,
            context: LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            active_column: Some("B".to_string()),
        };

        let lazy_frame = df!(
            "A" => &[1, 1, 1, 2, 2, 2, 3, 3, 3],
            "B" => &[1, 2, 3, 6, 5, 4, 8, 8, 8],)?
        .lazy();

        Ok((expr_domain, lazy_frame.groupby([col("A")])))
    }

    #[test]
    fn test_make_score_elts_expr_grouppy() -> Fallible<()> {
        let (_, lazy_frame) = get_groupby_test_data()?;

        let expr = col("B");
        let expr_make_score = make_score_elts_expr(expr, 0.2);

        // Get resulting scoring
        let frame_actual = lazy_frame
            .clone()
            .agg([expr_make_score])
            .sort("A", Default::default())
            .collect()?;

        // Get expected scoring
        let a = Series::new("A", &[1, 2, 3]);
        // let b = Series::new("B", &[Series::new("list", &[0.4, 1.4, 2.4]),
        //                         Series::new("list", &[2.4, 1.4, 0.4]),
        //                         Series::new("list", &[0.4, 0.4, 0.4])]); //without slice
        let b = Series::new(
            "B",
            &[
                Series::new("list", &[0.4, 1.4]),
                Series::new("list", &[0.4, 1.4]),
                Series::new("list", &[0.4, 0.4]),
            ],
        );

        let frame_expected = DataFrame::new(vec![a.clone(), b.clone()])?;

        // print to understand why not equal
        println!("Frame actual {}", frame_actual);
        println!("Frame expected {}", frame_expected);
        println!("Type of column actual: {:?}", frame_actual.dtypes());
        println!("Type of column expected: {:?}", frame_expected.dtypes());
        if let Ok(first_element) = frame_actual.column("B").unwrap().get(0) {
            println!("First element of Series B actual: {}", first_element);
        }
        if let Ok(first_element) = frame_expected.column("B").unwrap().get(0) {
            println!("First element of Series B expected: {}", first_element);
        }

        assert_eq!(frame_actual[0], a);
        assert_eq!(frame_actual[1], b);
        assert_eq!(frame_actual, frame_expected);
        Ok(())
    }
}
