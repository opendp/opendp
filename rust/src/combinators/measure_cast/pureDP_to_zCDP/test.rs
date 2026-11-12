use super::*;

#[test]
fn test_unbounded_privacy_loss() -> Fallible<()> {
    use crate::{domains::AtomDomain, measurements::make_laplace, metrics::AbsoluteDistance};
    let m = make_laplace(
        AtomDomain::<f64>::new_non_nan(),
        AbsoluteDistance::<f64>::default(),
        0.0,
        None,
    )?;
    let m = make_pureDP_to_zCDP(m)?;
    assert_eq!(m.map(&0.0)?, 0.0);
    assert_eq!(m.map(&1.0)?, f64::INFINITY);
    Ok(())
}
