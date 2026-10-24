use crate::domains::{CategoricalDomain, LazyFrameDomain, SeriesDomain, SeriesElementDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::make_stable_lazyframe;

use super::*;

fn assert_expr_to_physical<DI: 'static + SeriesElementDomain, DO: 'static + SeriesElementDomain>(
    in_elem_domain: DI,
    in_series: Series,
    out_elem_domain: DO,
    out_series: Series,
) -> Fallible<()> {
    let name = in_series.name().clone();
    let in_series_domain = SeriesDomain::new(name.clone(), in_elem_domain);
    let lf_domain = LazyFrameDomain::new(vec![in_series_domain.clone()])?;
    let lf = DataFrame::new(vec![in_series.into_column()])?.lazy();

    let t_binned = make_stable_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        lf.clone().with_column(col(name.clone()).to_physical()),
    )?;

    // check that data is transformed as expected
    let expected = DataFrame::new(vec![out_series.into_column()])?;
    let actual = t_binned.invoke(&lf)?.collect()?;
    assert_eq!(expected, actual);

    // check that domain is transformed as expected
    let out_series_domain = SeriesDomain::new(name, out_elem_domain);
    let expected_lf_domain = LazyFrameDomain::new(vec![out_series_domain])?;

    assert_eq!(t_binned.output_domain, expected_lf_domain);

    Ok(())
}

#[test]
fn test_expr_to_physical_categorical() -> Fallible<()> {
    let series = Series::new("data".into(), ["A", "B"]).cast(&DataType::Categorical(
        Categories::global(),
        Categories::global().mapping(),
    ))?;
    let lf = DataFrame::new(vec![series.into_column()])?.lazy();
    let domain = LazyFrameDomain::new(vec![SeriesDomain::new(
        "data",
        CategoricalDomain::default(),
    )])?;
    assert!(
        make_stable_lazyframe(
            domain,
            FrameDistance(SymmetricDistance),
            lf.with_column(col("data").to_physical()),
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn test_expr_to_physical_enum() -> Fallible<()> {
    let categories = ["A", "B", "C", "D"];
    let frozen = FrozenCategories::new(categories)?;
    let dtype = DataType::Enum(frozen.clone(), frozen.mapping().clone());
    let in_series = Series::new("data".into(), ["B", "A", "B", "C", "D"]).cast(&dtype)?;
    assert_expr_to_physical(
        crate::domains::EnumDomain::new(Series::new("".into(), categories))?,
        in_series,
        AtomDomain::<u32>::default(),
        Series::new("data".into(), [1u32, 0, 1, 2, 3]),
    )
}

#[test]
fn test_expr_to_physical_date() -> Fallible<()> {
    let days = [-1_i32, 0, 1, 19_723];
    assert_expr_to_physical(
        AtomDomain::<chrono::NaiveDate>::default(),
        Series::new("data".into(), days).cast(&DataType::Date)?,
        AtomDomain::<i32>::default(),
        Series::new("data".into(), days),
    )
}

#[test]
fn test_expr_to_physical_same() -> Fallible<()> {
    fn assert<D: 'static + SeriesElementDomain>(elem_domain: D, series: Series) -> Fallible<()> {
        assert_expr_to_physical(elem_domain.clone(), series.clone(), elem_domain, series)
    }
    assert(
        AtomDomain::<i32>::default(),
        Series::new("x".into(), [1i32, 4, 7, 3]),
    )?;
    assert(
        AtomDomain::<String>::default(),
        Series::new(
            "x".into(),
            ["C".to_string(), "A".to_string(), "B".to_string()],
        ),
    )?;
    assert(
        AtomDomain::<bool>::default(),
        Series::new("x".into(), [true, false, false]),
    )?;
    assert(
        AtomDomain::<f32>::default(),
        Series::new("x".into(), [1.0f32, 2.0, 7.0]),
    )?;
    Ok(())
}
