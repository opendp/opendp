use dashu::rbig;

use super::*;

#[test]
fn test_make_int_to_bigint() -> Fallible<()> {
    let space = (
        MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default()),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );

    let t_cast = make_int_to_bigint_threshold::<bool, i32, 2, f64>(space.clone())?;
    assert_eq!(
        t_cast.invoke(&HashMap::from([(false, i32::MIN), (true, i32::MAX)]))?,
        HashMap::from([(false, IBig::from(i32::MIN)), (true, IBig::from(i32::MAX))])
    );

    assert_eq!(t_cast.map(&(1, 1., 1.))?, (1, rbig!(1), rbig!(1)));
    assert!(t_cast.map(&(1, f64::NAN, 1.)).is_err());
    assert!(t_cast.map(&(1, 1., f64::NAN)).is_err());

    Ok(())
}

#[test]
fn test_make_noise_intexpfamily() -> Fallible<()> {
    let space = (
        MapDomain::new(AtomDomain::<bool>::default(), AtomDomain::<i32>::default()),
        L0PInfDistance(AbsoluteDistance::<f64>::default()),
    );

    assert!(
        IntExpFamily::<1> { scale: 1.0 }
            .make_noise_threshold(space.clone(), 0)
            .is_ok()
    );
    assert!(
        IntExpFamily::<1> { scale: f64::NAN }
            .make_noise_threshold(space.clone(), 0)
            .is_err()
    );

    Ok(())
}

#[test]
fn test_then_saturating_cast() -> Fallible<()> {
    //  with greater magnitude than MAX saturate to infinity
    let q = IBig::try_from(i8::MAX).unwrap();
    assert_eq!(i8::saturating_cast(q + IBig::ONE), i8::MAX);

    let f_i32 = then_saturating_cast_hashmap::<bool, i32>();
    assert_eq!(
        f_i32.eval(&HashMap::from([
            (false, IBig::from(i32::MAX) + IBig::ONE),
            (true, IBig::from(i32::MIN) - IBig::ONE)
        ]))?,
        HashMap::from([(false, i32::MAX), (true, i32::MIN)])
    );
    Ok(())
}
