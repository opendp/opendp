#[cfg(feature = "contrib")]
mod scan_csv;
#[cfg(feature = "contrib")]
pub use scan_csv::*;

mod make_collect;
pub use make_collect::*;

mod make_column;
pub use make_column::*;

mod make_lazy;
pub use make_lazy::*;

mod make_unpack;
pub use make_unpack::*;

#[cfg(feature = "contrib")]
mod write_csv;
#[cfg(feature = "contrib")]
pub use write_csv::*;
#[cfg(feature = "contrib")]
mod scan_parquet;
#[cfg(feature = "contrib")]
pub use scan_parquet::*;
#[cfg(feature = "contrib")]
mod sink_parquet;
#[cfg(feature = "contrib")]
pub use sink_parquet::*;

#[cfg(feature = "contrib")]
mod make_sum;
#[cfg(feature = "contrib")]
pub use make_sum::*;

#[cfg(feature = "contrib")]
mod make_group_by_stable;
#[cfg(feature = "contrib")]
pub use make_group_by_stable::*;
#[cfg(feature = "contrib")]
pub use make_group_by_stable::*;

#[cfg(feature = "contrib")]
mod make_col;
#[cfg(feature = "contrib")]
pub use make_col::*;

#[cfg(feature = "contrib")]
mod make_with_columns;
#[cfg(feature = "contrib")]
pub use make_with_columns::*;

#[cfg(feature = "contrib")]
mod make_agg_trans;
#[cfg(feature = "contrib")]
pub use make_agg_trans::*;
#[cfg(feature = "contrib")]
mod make_clamp;
#[cfg(feature = "contrib")]
pub use make_clamp::*;

#[cfg(feature = "contrib")]
mod make_quantile_score_candidates_expr;
#[cfg(feature = "contrib")]
pub use make_quantile_score_candidates_expr::*;

// #[cfg(feature = "contrib")]
// mod make_quantile_scores_continuous_expr;
// #[cfg(feature = "contrib")]
// pub use make_quantile_scores_continuous_expr::*;

#[cfg(test)]
pub mod polars_test {
    use crate::domains::{
        AtomDomain, ExprDomain, LazyFrameContext, LazyFrameDomain, LazyGroupByContext,
        LazyGroupByDomain, SeriesDomain,
    };
    use crate::error::*;
    use polars::prelude::*;

    pub fn get_select_test_data() -> Fallible<(ExprDomain<LazyFrameDomain>, Arc<LazyFrame>)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::new_closed((0.5, 2.5))?),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?
        .with_counts(df!["count" => [3u32]]?.lazy())?
        .with_counts(df!["A" => [1, 2], "count" => [1u32, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2u32, 1]]?.lazy())?
        .with_counts(df!["C" => [8, 9, 10], "count" => [1u32, 1, 1]]?.lazy())?;

        let expr_domain = ExprDomain::new(
            frame_domain.clone(),
            LazyFrameContext::Select,
            Some("B".to_string()),
        );

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)?
        .lazy();

        Ok((expr_domain, Arc::new(lazy_frame)))
    }

    pub fn get_grouped_test_data() -> Fallible<(ExprDomain<LazyGroupByDomain>, Arc<LazyGroupBy>)> {
        let (expr_domain, lazy_frame) = get_select_test_data()?;
        let expr_domain = ExprDomain::new(
            expr_domain.lazy_frame_domain,
            LazyGroupByContext {
                columns: vec!["A".to_string()],
            },
            expr_domain.active_column,
        );

        Ok((
            expr_domain,
            Arc::new((*lazy_frame).clone().group_by_stable([col("A")])),
        ))
    }
}
