use super::*;

#[test]
fn test_l01inf_to_lp() -> Fallible<()> {
    use crate::traits::CastInternalRational;
    use dashu::rational::RBig;

    assert_eq!(l01inf_to_lp((3, 5, 2), 2)?, 3.);
    assert_eq!(l01inf_to_lp((2, 5, 2), 1)?, 4.);
    assert_eq!(l01inf_to_lp((3, 5, 0), 2)?, 0.);
    let bound = l01inf_to_lp((2, 2, 1), 2)?.into_rational()?;
    assert!(&bound * &bound >= RBig::from(2));
    Ok(())
}
