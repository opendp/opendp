use crate::domains::{AtomDomain, LazyFrameDomain, SeriesDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::make_stable_lazyframe;

use super::*;

#[test]
fn test_expr_cut() -> Fallible<()> {
    let mut series_domain = SeriesDomain::new("data", AtomDomain::<i32>::default());
    let lf_domain = LazyFrameDomain::new(vec![series_domain.clone()])?;
    let lf = df!["data" => &[-2i32, -1, 0, 1, 2]]?.lazy();

    let expr = col("data").cut(vec![-1.0, 1.0], None::<Vec<String>>, false, false);
    let t_cut = make_stable_lazyframe(lf_domain, SymmetricDistance, lf.clone().with_column(expr))?;

    // check data output
    let actual = t_cut
        .invoke(&lf)?
        .with_column(all().cast(DataType::String))
        .collect()?;
    let expected = df!["data" => ["(-inf, -1]", "(-inf, -1]", "(-1, 1]", "(-1, 1]", "(1, inf]"]]?;
    assert_eq!(expected, actual);

    // check domain output
    let encoding = compute_labels(&[-1.0, 1.0], false)?;
    series_domain.set_element_domain(CategoricalDomain::new_with_categories(encoding)?);
    let lf_domain_exp = LazyFrameDomain::new(vec![series_domain])?;

    assert_eq!(t_cut.output_domain, lf_domain_exp);

    Ok(())
}
