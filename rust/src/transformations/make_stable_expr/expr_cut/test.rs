use crate::domains::{AtomDomain, EnumDomain, LazyFrameDomain, SeriesDomain};
use crate::metrics::{FrameDistance, SymmetricDistance};
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_expr_cut() -> Fallible<()> {
    let mut series_domain = SeriesDomain::new("data", AtomDomain::<i32>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain.clone()])?;
    let lf = df!["data" => &[-2i32, -1, 0, 1, 2]]?.lazy();

    let expr = col("data").cut(vec![-1.0, 1.0], None::<Vec<String>>, false, false);
    let t_cut = make_stable_lazyframe(
        lf_domain,
        FrameDistance(SymmetricDistance),
        lf.clone().with_column(expr),
    )?;

    // check data output
    let actual = t_cut
        .invoke(&lf)?
        .with_column(all().as_expr().cast(DataType::String))
        .collect()?;
    let expected = df!["data" => ["(-inf, -1]", "(-inf, -1]", "(-1, 1]", "(-1, 1]", "(1, inf]"]]?;
    assert_eq!(expected, actual);

    // check domain output
    let encoding = compute_labels(&[-1.0, 1.0], false)?;
    series_domain.set_element_domain(EnumDomain::new(Series::new(
        "".into(),
        encoding.iter().map(|v| v.as_str()).collect::<Vec<_>>(),
    ))?);
    series_domain.nullable = true;
    let lf_domain_exp = LazyFrameDomain::new(vec![series_domain])?;

    assert_eq!(t_cut.output_domain, lf_domain_exp);

    Ok(())
}

#[test]
fn test_cut_physical_codes_independent_of_rows() -> Fallible<()> {
    let x = df!["data" => vec![0.5f64; 100]]?.lazy();
    let y = df!["data" => [vec![2.0], vec![0.5; 100]].concat()]?.lazy();
    let t_cut = make_stable_lazyframe(
        LazyFrameDomain::new(vec![SeriesDomain::new(
            "data",
            AtomDomain::<f64>::default(),
        )])?,
        FrameDistance(SymmetricDistance),
        x.clone().select([col("data")
            .cut(vec![1.0], None::<Vec<String>>, false, false)
            .to_physical()]),
    )?;
    let x_codes = t_cut.invoke(&x)?.collect()?;
    let y_codes = t_cut.invoke(&y)?.collect()?;
    assert_eq!(
        x_codes
            .column("data")?
            .u32()?
            .into_no_null_iter()
            .collect::<Vec<_>>(),
        vec![0; 100]
    );
    assert_eq!(
        y_codes
            .column("data")?
            .u32()?
            .into_no_null_iter()
            .collect::<Vec<_>>(),
        [vec![1], vec![0; 100]].concat()
    );
    Ok(())
}

#[test]
fn test_cut_sorts_breaks_before_deriving_enum_labels() -> Fallible<()> {
    let lf = df!["data" => [-2i32, 0, 2]]?.lazy();
    let t_cut = make_stable_lazyframe(
        LazyFrameDomain::new(vec![SeriesDomain::new(
            "data",
            AtomDomain::<i32>::default(),
        )])?,
        FrameDistance(SymmetricDistance),
        lf.clone().select([col("data")
            .cut(vec![1.0, -1.0], None::<Vec<String>>, false, false)
            .to_physical()]),
    )?;
    let codes = t_cut.invoke(&lf)?.collect()?;
    assert_eq!(
        codes
            .column("data")?
            .u32()?
            .into_no_null_iter()
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    Ok(())
}
