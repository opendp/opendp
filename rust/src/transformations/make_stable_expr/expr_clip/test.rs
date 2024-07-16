use crate::domains::{AtomDomain, DslPlanDomain};
use crate::metrics::SymmetricDistance;
use crate::transformations::test_helper::get_test_data;

use super::*;

#[test]
fn test_make_expr_clip() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let lp = lf.logical_plan;
    let expr_domain = lf_domain.clone().select();

    let expected = col("const_1f64").clip(lit(0.), lit(0.5));

    let t_clip = expected
        .clone()
        .make_stable(expr_domain, SymmetricDistance)?;
    let actual = t_clip.invoke(&(lp, all()))?.1;

    assert_eq!(expected, actual);

    let mut series_domain = lf_domain
        .series_domains
        .into_iter()
        .find(|s| s.field.name.as_str() == "const_1f64")
        .unwrap();
    series_domain.element_domain = Arc::new(AtomDomain::<f64>::new_closed((0.0, 0.5))?);

    let mut lf_domain_exp = DslPlanDomain::new(vec![series_domain])?;
    lf_domain_exp.margins = t_clip.output_domain.frame_domain.margins.clone();

    assert_eq!(t_clip.output_domain, lf_domain_exp.select());

    Ok(())
}
