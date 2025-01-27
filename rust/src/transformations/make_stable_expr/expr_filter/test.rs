use polars::df;
use polars::prelude::{col, lit, IntoLazy, NamedFrom};
use polars::series::Series;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::measurements::make_private_lazyframe;
use crate::measures::MaxDivergence;
use crate::metrics::SymmetricDistance;
use crate::polars::PrivacyNamespace;

use super::*;

#[test]
fn make_expr_filter_standard() -> Fallible<()> {
    let a_domain = SeriesDomain::new("a", AtomDomain::<i32>::default());
    let b_domain = SeriesDomain::new("b", AtomDomain::<i32>::default());
    let lf_domain = LazyFrameDomain::new(vec![a_domain, b_domain])?;
    let lf = df!("a" => &[1, 2, 3], "b" => &[10, 20, 30])?.lazy();

    let lf_filter = lf
        .clone()
        .select([col("a").filter(col("b").gt(15)).dp().count(None)]);

    let t_obs = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf_filter,
        Some(0.0),
        None,
    )?;
    let df_observed = t_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("a")?.as_materialized_series(),
        &Series::new("a".into(), [2])
    );
    Ok(())
}

#[test]
fn make_expr_filter_impute() -> Fallible<()> {
    let series_domain = SeriesDomain::new("", OptionDomain::new(AtomDomain::<i32>::default()));
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?
        .with_margin(Margin::default().with_max_partition_length(5))?;
    let lf = df!("" => &[Some(1), Some(2), Some(3), None])?.lazy();

    let lf_filter = lf.clone().select([col("")
        .filter(col("").is_not_null())
        .fill_null(lit(0))
        .dp()
        .sum((1, 3), None)]);

    let t_obs = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf_filter,
        Some(0.0),
        None,
    )?;
    let df_observed = t_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("")?.as_materialized_series(),
        &Series::new("".into(), [6])
    );
    Ok(())
}
