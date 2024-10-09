use crate::error::Fallible;

use super::*;

#[test]
fn test_exponential() -> Fallible<()> {
    let input_domain = VectorDomain::new(AtomDomain::default());
    let input_metric = LInfDistance::default();
    let de = make_report_noisy_max_gumbel(input_domain, input_metric, 1., Optimize::Max)?;
    let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
    println!("{:?}", release);

    Ok(())
}

#[test]
fn test_max_vs_min() -> Fallible<()> {
    assert_eq!(
        select_score([1, 2, 3].into_iter(), Optimize::Max, FBig::ZERO)?,
        2
    );
    assert_eq!(
        select_score([1, 2, 3].into_iter(), Optimize::Min, FBig::ZERO)?,
        0
    );

    assert_eq!(
        select_score([1, 1, 100_000].into_iter(), Optimize::Max, FBig::ONE)?,
        2
    );
    assert_eq!(
        select_score([1, 100_000, 100_000].into_iter(), Optimize::Min, FBig::ONE)?,
        0
    );
    Ok(())
}
