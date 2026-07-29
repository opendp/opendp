use super::*;

fn assert_contains<Bk: IntervalBackend>(
    interval: &Interval<Bk>,
    lower: f64,
    upper: f64,
) -> Fallible<()> {
    assert!(
        interval.lower_f64()? <= lower,
        "lower endpoint {} does not contain {lower}",
        interval.lower_f64()?
    );
    assert!(
        interval.upper_f64()? >= upper,
        "upper endpoint {} does not contain {upper}",
        interval.upper_f64()?
    );
    Ok(())
}

fn check_arithmetic<Bk>() -> Fallible<()>
where
    Bk: IntervalArithmeticBackend,
{
    assert_contains(
        &Interval::<Bk>::between(1.0, 2.0)?.add(Interval::between(3.0, 4.0)?)?,
        4.0,
        6.0,
    )?;
    assert_contains(
        &Interval::<Bk>::between(1.0, 2.0)?.sub(Interval::between(3.0, 4.0)?)?,
        -3.0,
        -1.0,
    )?;
    assert_contains(&Interval::<Bk>::between(1.0, 2.0)?.neg()?, -2.0, -1.0)?;

    // Exercise every sign-specific multiplication arm and the four-corner fallback.
    for (lhs, rhs, expected) in [
        ((1.0, 2.0), (3.0, 4.0), (3.0, 8.0)),
        ((-2.0, -1.0), (-4.0, -3.0), (3.0, 8.0)),
        ((1.0, 2.0), (-4.0, -3.0), (-8.0, -3.0)),
        ((-2.0, -1.0), (3.0, 4.0), (-8.0, -3.0)),
        ((-2.0, 3.0), (-4.0, 5.0), (-12.0, 15.0)),
    ] {
        let product =
            Interval::<Bk>::between(lhs.0, lhs.1)?.mul(Interval::between(rhs.0, rhs.1)?)?;
        assert_contains(&product, expected.0, expected.1)?;
    }

    assert_contains(
        &Interval::<Bk>::between(2.0, 4.0)?.div(Interval::between(1.0, 2.0)?)?,
        1.0,
        4.0,
    )?;
    assert_contains(&Interval::<Bk>::between(-3.0, 2.0)?.abs()?, 0.0, 3.0)?;
    assert!(Interval::<Bk>::between(-1.0, 1.0)?.recip().is_err());
    Ok(())
}

#[test]
fn test_interval_arithmetic_backends() -> Fallible<()> {
    check_arithmetic::<A>()?;
    check_arithmetic::<B>()?;
    check_arithmetic::<C>()?;
    check_arithmetic::<D>()
}

#[test]
fn test_interval_construction_and_clamping() -> Fallible<()> {
    let widened = CInterval::from_approx(1.0)?;
    assert!(widened.lower_f64()? < 1.0);
    assert!(widened.upper_f64()? > 1.0);

    assert!(CInterval::between(2.0, 1.0).is_err());
    assert!(CInterval::point(f64::INFINITY).is_err());
    assert!(CInterval::between(-1.0, 2.0)?.contains_zero()?);
    assert!(CInterval::between(0.0, 2.0)?.is_nonnegative()?);
    assert!(CInterval::between(-2.0, 0.0)?.is_nonpositive()?);

    assert_contains(&CInterval::between(-2.0, 3.0)?.clamp01()?, 0.0, 1.0)?;
    assert!(CInterval::point(0.5)?.clamp(1.0, 0.0).is_err());
    Ok(())
}

fn check_transcendentals<Bk>() -> Fallible<()>
where
    Bk: IntervalExpBackend,
{
    assert_contains(
        &Interval::<Bk>::point(1.0)?.exp()?,
        std::f64::consts::E,
        std::f64::consts::E,
    )?;
    assert_contains(
        &Interval::<Bk>::point(1.0)?.exp_m1()?,
        std::f64::consts::E - 1.0,
        1.0_f64.exp_m1(),
    )?;
    assert_contains(&Interval::<Bk>::point(1.0)?.ln()?, 0.0, 0.0)?;
    assert_contains(&Interval::<Bk>::point(4.0)?.sqrt()?, 2.0, 2.0)?;
    assert!(Interval::<Bk>::between(-1.0, 1.0)?.ln().is_err());
    assert!(Interval::<Bk>::between(-1.0, 1.0)?.sqrt().is_err());
    Ok(())
}

#[test]
fn test_interval_transcendental_backends() -> Fallible<()> {
    // Approximate arithmetic makes no containment guarantee, but should expose
    // the native results without widening.
    let approximate = AInterval::point(1.0)?.exp_m1()?;
    assert_eq!(approximate.lower_f64()?, 1.0_f64.exp_m1());
    assert_eq!(approximate.upper_f64()?, 1.0_f64.exp_m1());

    check_transcendentals::<B>()?;
    check_transcendentals::<D>()
}

#[test]
fn test_best_effort_error_functions() -> Fallible<()> {
    assert_contains(&BInterval::point(0.0)?.erfc()?, 1.0, 1.0)?;
    assert_contains(&BInterval::point(0.0)?.erfcx()?, 1.0, 1.0)?;
    assert_contains(&BInterval::point(1.0)?.erfc_inv()?, 0.0, 0.0)?;

    assert!(BInterval::between(0.0, 1.0)?.erfc_inv().is_err());
    assert!(BInterval::between(1.0, 2.0)?.erfc_inv().is_err());
    Ok(())
}
