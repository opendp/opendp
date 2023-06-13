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
mod make_col;
#[cfg(feature = "contrib")]
pub use make_col::*;

#[cfg(feature = "contrib")]
mod make_select_trans;
#[cfg(feature = "contrib")]
pub use make_select_trans::*;
#[cfg(feature = "contrib")]
mod make_groupby;
#[cfg(feature = "contrib")]
pub use make_groupby::*;
#[cfg(feature = "contrib")]
mod make_agg_trans;
#[cfg(feature = "contrib")]
pub use make_agg_trans::*;
#[cfg(feature = "contrib")]
mod make_clamp;
#[cfg(feature = "contrib")]
pub use make_clamp::*;

#[cfg(test)]
mod test {
    use crate::domains::{
        AtomDomain, ExprDomain, LazyFrameContext, LazyFrameDomain, SeriesDomain,
    };
    use crate::error::*;
    use polars::prelude::*;

    pub fn get_test_data() -> Fallible<(ExprDomain<LazyFrameContext>, LazyFrame, LazyFrameDomain)> {
        let frame_domain = LazyFrameDomain::new(vec![
            SeriesDomain::new("A", AtomDomain::<i32>::default()),
            SeriesDomain::new("B", AtomDomain::<f64>::default()),
            SeriesDomain::new("C", AtomDomain::<i32>::default()),
        ])?
        .with_counts(df!["A" => [1, 2], "count" => [1, 2]]?.lazy())?
        .with_counts(df!["B" => [1.0, 2.0], "count" => [2, 1]]?.lazy())?
        .with_counts(df!["C" => [8, 9, 10], "count" => [1, 1, 1]]?.lazy())?;

        let expr_domain = ExprDomain {
            lazy_frame_domain: frame_domain.clone(),
            context: LazyFrameContext::Select,
            active_column: None,
        };

        let lazy_frame = df!(
            "A" => &[1, 2, 2],
            "B" => &[1.0, 1.0, 2.0],
            "C" => &[8, 9, 10],)?
        .lazy();

        Ok((expr_domain, lazy_frame, frame_domain))
    }
}
