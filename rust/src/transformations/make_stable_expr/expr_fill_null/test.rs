use crate::domains::LogicalPlanDomain;
use crate::metrics::SymmetricDistance;
use crate::transformations::test_helper::get_test_data;

use super::*;

#[test]
fn test_make_expr_fill_null() -> Fallible<()> {
    let (lf_domain, lf) = get_test_data()?;
    let lp = lf.logical_plan;
    let expr_domain = lf_domain.clone().select();

    let expected = col("cycle_(..100i32)").fill_null(0);

    let t_clip = expected
        .clone()
        .make_stable(expr_domain, SymmetricDistance)?;
    let actual = t_clip.invoke(&(lp, all()))?.1;

    assert_eq!(expected, actual);

    let mut series_domain = lf_domain
        .series_domains
        .into_iter()
        .find(|s| s.field.name.as_str() == "cycle_(..100i32)")
        .unwrap();
    series_domain.nullable = false;

    let mut lf_domain_exp = LogicalPlanDomain::new(vec![series_domain])?;
    lf_domain_exp.margins = t_clip.output_domain.frame_domain.margins.clone();

    assert_eq!(t_clip.output_domain, lf_domain_exp.select());

    Ok(())
}
