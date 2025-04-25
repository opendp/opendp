use crate::{domains::AtomDomain, error::Fallible, metrics::AbsoluteDistance};

use super::make_canonical_noise;

#[test]
fn test_canonical_noise() -> Fallible<()> {
    let m_cnd = make_canonical_noise(
        AtomDomain::new_non_nan(),
        AbsoluteDistance::default(),
        1.,
        (0.1, 1e-7),
    )?;
    assert!(m_cnd.invoke(&1.).is_ok());
    assert_eq!(m_cnd.map(&1.)?, (0.1, 1e-7));
    Ok(())
}
