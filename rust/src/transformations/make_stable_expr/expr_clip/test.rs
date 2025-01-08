use crate::domains::AtomDomain;
use crate::metrics::SymmetricDistance;
use crate::transformations::test_helper::get_test_data;

use super::*;

#[test]
fn test_make_expr_clip() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;

    let expected = col("const_1f64").clip(lit(0.), lit(0.5));

    let t_clip = expected
        .clone()
        .make_stable(lf_domain.clone().select(), SymmetricDistance)?;
    let actual = t_clip.invoke(&lf.logical_plan)?.expr;

    assert_eq!(expected, actual);

    let mut series_domain = lf_domain
        .series_domains
        .into_iter()
        .find(|s| s.name.as_str() == "const_1f64")
        .unwrap();
    series_domain.set_element_domain(AtomDomain::<f64>::new_closed((0.0, 0.5))?);

    let lf_domain_exp = ExprDomain {
        column: series_domain,
        context: t_clip.output_domain.context.clone(),
    };

    assert_eq!(t_clip.output_domain, lf_domain_exp);

    Ok(())
}
