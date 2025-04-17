use dashu::rbig;

use crate::metrics::L2Distance;

use super::*;

#[test]
fn test_make_int_to_bigint() -> Fallible<()> {
    let space = (
        VectorDomain::new(AtomDomain::<i32>::default()),
        L2Distance::<f64>::default(),
    );

    let t_cast = make_int_to_bigint::<i32, 2, f64>(space.clone())?;
    assert_eq!(t_cast.invoke(&vec![i32::MIN])?, vec![IBig::from(i32::MIN)]);
    assert_eq!(t_cast.invoke(&vec![i32::MAX])?, vec![IBig::from(i32::MAX)]);
    assert_eq!(t_cast.invoke(&vec![0])?, vec![IBig::from(0)]);

    assert_eq!(t_cast.map(&1.)?, rbig!(1));
    assert!(t_cast.map(&f64::NAN).is_err());

    Ok(())
}

#[test]
fn test_make_noise_intexpfamily() -> Fallible<()> {
    let space = (
        AtomDomain::<i32>::default(),
        AbsoluteDistance::<f64>::default(),
    );

    assert!(
        IntExpFamily::<1> { scale: 1.0 }
            .make_noise(space.clone())
            .is_ok()
    );
    assert!(
        IntExpFamily::<1> { scale: f64::NAN }
            .make_noise(space.clone())
            .is_err()
    );

    Ok(())
}

#[test]
fn test_then_saturating_cast() -> Fallible<()> {
    //  with greater magnitude than MAX saturate to infinity
    let q = IBig::try_from(i8::MAX).unwrap();
    assert_eq!(i8::saturating_cast(q + IBig::ONE), i8::MAX);

    let f_i32 = then_saturating_cast::<i32>();
    assert_eq!(
        f_i32.eval(&vec![IBig::from(i32::MAX) + IBig::ONE])?,
        vec![i32::MAX]
    );
    assert_eq!(
        f_i32.eval(&vec![IBig::from(i32::MIN) - IBig::ONE])?,
        vec![i32::MIN]
    );
    Ok(())
}
