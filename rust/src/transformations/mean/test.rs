use crate::metrics::SymmetricDistance;

use super::*;

#[test]
fn test_make_bounded_mean_symmetric() -> Fallible<()> {
    let transformation = make_mean(
        VectorDomain::new(AtomDomain::new_closed((0., 10.))?).with_size(5),
        SymmetricDistance::default(),
    )?;
    let arg = vec![1., 2., 3., 4., 5.];
    let ret = transformation.invoke(&arg)?;
    let expected = 3.;
    assert_eq!(ret, expected);
    assert!(transformation.check(&1, &2.)?);

    Ok(())
}
