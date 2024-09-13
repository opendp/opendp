use crate::domains::{LazyFrameDomain, SeriesDomain, SeriesElementDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

use super::*;

fn assert_expr_to_physical<DI: 'static + SeriesElementDomain, DO: 'static + SeriesElementDomain>(
    in_elem_domain: DI,
    in_series: Series,
    out_elem_domain: DO,
    out_series: Series,
) -> Fallible<()> {
    let name = in_series.name().to_string();
    let in_series_domain = SeriesDomain::new(&name, in_elem_domain);
    let lf_domain = LazyFrameDomain::new(vec![in_series_domain.clone()])?;
    let lf = DataFrame::new(vec![in_series])?.lazy();

    let t_binned = make_stable_lazyframe(
        lf_domain,
        SymmetricDistance,
        lf.clone().with_column(col(&name).to_physical()),
    )?;

    // check that data is transformed as expected
    let expected = DataFrame::new(vec![out_series])?;
    let actual = t_binned.invoke(&lf)?.collect()?;
    assert_eq!(expected, actual);

    // check that domain is transformed as expected
    let out_series_domain = SeriesDomain::new(&name, out_elem_domain);
    let expected_lf_domain = LazyFrameDomain::new(vec![out_series_domain])?;

    assert_eq!(t_binned.output_domain, expected_lf_domain);

    Ok(())
}

#[test]
fn test_expr_to_physical_categorical() -> Fallible<()> {
    let in_elem_domain = CategoricalDomain::new_with_encoding(
        vec!["A", "B", "C", "D"]
            .into_iter()
            .map(ToString::to_string)
            .collect(),
    )?;

    let in_series = Series::new("data", ["A", "B", "B", "C", "D"])
        .cast(&DataType::Categorical(None, Default::default()))?;

    let out_elem_domain = AtomDomain::<u32>::default();
    let out_series = Series::new("data", [0u32, 1, 1, 2, 3]);
    assert_expr_to_physical(in_elem_domain, in_series, out_elem_domain, out_series)
}

#[test]
fn test_expr_to_physical_same() -> Fallible<()> {
    fn assert<D: 'static + SeriesElementDomain>(elem_domain: D, series: Series) -> Fallible<()> {
        assert_expr_to_physical(elem_domain.clone(), series.clone(), elem_domain, series)
    }
    assert(
        AtomDomain::<i32>::default(),
        Series::new("x", [1i32, 4, 7, 3]),
    )?;
    assert(
        AtomDomain::<String>::default(),
        Series::new("x", ["C".to_string(), "A".to_string(), "B".to_string()]),
    )?;
    assert(
        AtomDomain::<bool>::default(),
        Series::new("x", [true, false, false]),
    )?;
    assert(
        AtomDomain::<f32>::default(),
        Series::new("x", [1.0f32, 2.0, 7.0]),
    )?;
    Ok(())
}
