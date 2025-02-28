use polars::df;
use polars::prelude::{col, IntoLazy, NamedFrom};
use polars::series::Series;

use crate::domains::{AtomDomain, LazyFrameDomain, OptionDomain, SeriesDomain};
use crate::measurements::make_private_lazyframe;
use crate::measures::MaxDivergence;
use crate::metrics::SymmetricDistance;
use crate::polars::PrivacyNamespace;

use super::*;

#[test]
fn make_expr_drop_nan_standard() -> Fallible<()> {
    let series_domain = SeriesDomain::new("", AtomDomain::<f32>::new_nullable());
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?
        .with_margin(Margin::default().with_max_partition_length(5))?;
    let lf = df!("" => &[1.0, f32::NAN])?.lazy();

    let lf_filter = lf
        .clone()
        .select([col("").drop_nans().dp().sum((0.0, 1.0), None)]);

    let m_obs = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf_filter,
        Some(0.0),
        None,
    )?;

    let df_observed = m_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("")?.as_materialized_series(),
        &Series::new("".into(), [1.0])
    );
    Ok(())
}

#[test]
fn make_expr_drop_null_standard() -> Fallible<()> {
    let series_domain = SeriesDomain::new("", OptionDomain::new(AtomDomain::<i32>::default()));
    let lf_domain = LazyFrameDomain::new(vec![series_domain])?
        .with_margin(Margin::default().with_max_partition_length(5))?;
    let lf = df!("" => &[Some(1i32), None])?.lazy();

    let lf_filter = lf
        .clone()
        .select([col("").drop_nulls().dp().sum((0, 1), None)]);

    let m_obs = make_private_lazyframe(
        lf_domain,
        SymmetricDistance,
        MaxDivergence,
        lf_filter,
        Some(0.0),
        None,
    )?;

    let df_observed = m_obs.invoke(&lf)?.collect()?;
    assert_eq!(
        df_observed.column("")?.as_materialized_series(),
        &Series::new("".into(), [1i32])
    );
    Ok(())
}
