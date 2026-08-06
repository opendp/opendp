use super::*;
use crate::{error::ErrorVariant, traits::ToFloatRounded};
use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
};

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

fn check_arithmetic<Bk: IntervalBackend>() -> Fallible<()> {
    assert_contains(
        &(Interval::<Bk>::between(1.0, 2.0)? + Interval::between(3.0, 4.0)?)?,
        4.0,
        6.0,
    )?;
    assert_contains(
        &(Interval::<Bk>::between(1.0, 2.0)? - Interval::between(3.0, 4.0)?)?,
        -3.0,
        -1.0,
    )?;
    assert_contains(&(-Interval::<Bk>::between(1.0, 2.0)?)?, -2.0, -1.0)?;

    for (lhs, rhs, expected) in [
        ((1.0, 2.0), (3.0, 4.0), (3.0, 8.0)),
        ((-2.0, -1.0), (-4.0, -3.0), (3.0, 8.0)),
        ((1.0, 2.0), (-4.0, -3.0), (-8.0, -3.0)),
        ((-2.0, -1.0), (3.0, 4.0), (-8.0, -3.0)),
        ((-2.0, 3.0), (-4.0, 5.0), (-12.0, 15.0)),
    ] {
        let product = (Interval::<Bk>::between(lhs.0, lhs.1)? * Interval::between(rhs.0, rhs.1)?)?;
        assert_contains(&product, expected.0, expected.1)?;
    }

    assert_contains(
        &(Interval::<Bk>::between(2.0, 4.0)? / Interval::between(1.0, 2.0)?)?,
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
    check_arithmetic::<S<backend::Dashu>>()?;
    Ok(())
}

fn check_native_finite_overflow<Bk: IntervalBackend>() -> Fallible<()> {
    let positive = (Interval::<Bk>::point(f64::MAX)? + Interval::point(f64::MAX)?)?;
    assert_eq!(positive.lower_f64()?, f64::MAX);
    assert_eq!(positive.upper_f64()?, f64::INFINITY);

    let negative = (Interval::<Bk>::point(-f64::MAX)? + Interval::point(-f64::MAX)?)?;
    assert_eq!(negative.lower_f64()?, f64::NEG_INFINITY);
    assert_eq!(negative.upper_f64()?, -f64::MAX);
    Ok(())
}

#[test]
fn test_native_finite_overflow_is_directionally_enclosed() -> Fallible<()> {
    check_native_finite_overflow::<B>()?;
    check_native_finite_overflow::<C>()?;

    let positive = (AInterval::point(f64::MAX)? + AInterval::point(f64::MAX)?)?;
    assert_eq!(positive.lower_f64()?, f64::INFINITY);
    assert_eq!(positive.upper_f64()?, f64::INFINITY);

    let negative = (AInterval::point(-f64::MAX)? + AInterval::point(-f64::MAX)?)?;
    assert_eq!(negative.lower_f64()?, f64::NEG_INFINITY);
    assert_eq!(negative.upper_f64()?, f64::NEG_INFINITY);
    Ok(())
}

#[test]
fn test_special_values_and_numeric_domains() -> Fallible<()> {
    let positive_infinity = SInterval::<backend::Dashu>::point(f64::INFINITY)?;
    let negative_infinity = SInterval::<backend::Dashu>::point(f64::NEG_INFINITY)?;
    let zero = SInterval::<backend::Dashu>::point(0.0)?;

    assert_eq!(
        (positive_infinity.clone() + negative_infinity)
            .unwrap_err()
            .variant,
        crate::error::ErrorVariant::NumericIndeterminate
    );
    assert_eq!(
        (zero * positive_infinity.clone()).unwrap_err().variant,
        crate::error::ErrorVariant::NumericIndeterminate
    );
    assert_eq!(
        (positive_infinity.clone() / positive_infinity)
            .unwrap_err()
            .variant,
        crate::error::ErrorVariant::NumericIndeterminate
    );
    assert_eq!(
        SInterval::<backend::Dashu>::point(-1.0)?
            .ln()
            .unwrap_err()
            .variant,
        crate::error::ErrorVariant::NumericDomain
    );
    assert_eq!(
        SInterval::<backend::Dashu>::point(-1.0)?
            .sqrt()
            .unwrap_err()
            .variant,
        crate::error::ErrorVariant::NumericDomain
    );
    Ok(())
}

#[test]
fn test_interval_construction_and_clamping() -> Fallible<()> {
    let widened = CInterval::from_approx(1.0)?;
    assert!(widened.lower_f64()? < 1.0);
    assert!(widened.upper_f64()? > 1.0);

    assert!(CInterval::between(2.0, 1.0).is_err());
    assert!(CInterval::point(f64::NAN).is_err());
    assert_eq!(CInterval::point(f64::INFINITY)?.lower_f64()?, f64::INFINITY);
    assert!(CInterval::between(-1.0, 2.0)?.contains_zero()?);
    assert!(CInterval::between(0.0, 2.0)?.is_nonnegative()?);
    assert!(CInterval::between(-2.0, 0.0)?.is_nonpositive()?);

    assert_contains(&CInterval::between(-2.0, 3.0)?.clamp01()?, 0.0, 1.0)?;
    assert!(CInterval::point(0.5)?.clamp(1.0, 0.0).is_err());

    assert_contains(
        &CInterval::between(1.0, 3.0)?.min(CInterval::between(2.0, 4.0)?)?,
        1.0,
        3.0,
    )?;
    assert_contains(
        &CInterval::between(1.0, 3.0)?.max(CInterval::between(2.0, 4.0)?)?,
        2.0,
        4.0,
    )?;
    assert_eq!(
        CInterval::between(-2.0, 3.0)?
            .abs_upper()?
            .to_f64(Direction::Up)?,
        3.0
    );
    Ok(())
}

fn check_transcendentals<Bk>() -> Fallible<()>
where
    Bk: IntervalBackend,
    Bk::Scalar: DirectedTranscendental,
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
    assert_eq!(
        Interval::<Bk>::between(-1.0, 1.0)?
            .ln()
            .unwrap_err()
            .variant,
        ErrorVariant::NumericDomain
    );
    assert_eq!(
        Interval::<Bk>::between(-1.0, 1.0)?
            .sqrt()
            .unwrap_err()
            .variant,
        ErrorVariant::NumericDomain
    );

    let boundary = Interval::<Bk>::between(0.0, 1.0)?.ln()?;
    assert_eq!(boundary.lower_f64()?, f64::NEG_INFINITY);
    assert!(boundary.upper_f64()? >= 0.0);
    Ok(())
}

#[test]
fn test_interval_transcendental_backends() -> Fallible<()> {
    let approximate = AInterval::point(1.0)?.exp_m1()?;
    assert_eq!(approximate.lower_f64()?, 1.0_f64.exp_m1());
    assert_eq!(approximate.upper_f64()?, 1.0_f64.exp_m1());

    check_transcendentals::<B>()?;
    check_transcendentals::<S<backend::Dashu>>()?;
    Ok(())
}

fn high_precision_exp_bounds(x: f64) -> Fallible<(f64, f64)> {
    let lower = FBig::<Down>::try_from(x)?
        .with_precision(1024)
        .value()
        .exp();
    let upper = FBig::<Up>::try_from(x)?.with_precision(1024).value().exp();
    let minimum = RBig::try_from(f64::from_bits(1))?;
    let next_minimum = RBig::try_from(f64::from_bits(2))?;

    if RBig::try_from(upper.clone())? < minimum {
        return Ok((0.0, f64::from_bits(1)));
    }
    if RBig::try_from(lower.clone())? > minimum && RBig::try_from(upper.clone())? < next_minimum {
        return Ok((f64::from_bits(1), f64::from_bits(2)));
    }

    Ok((lower.to_f64_rounded(), upper.to_f64_rounded()))
}

#[test]
fn test_interval_error_propagation() -> Fallible<()> {
    let positive = SInterval::<backend::Dashu>::point(10_000.0)?.exp()?;
    assert_eq!(
        positive.upper_f64().unwrap_err().variant,
        ErrorVariant::NumericRangeAbove
    );

    let negative = (-positive)?;
    assert_eq!(
        negative.lower_f64().unwrap_err().variant,
        ErrorVariant::NumericRangeBelow
    );

    assert_eq!(
        SInterval::<backend::Dashu>::between(-1.0, 1.0)?
            .recip()
            .unwrap_err()
            .variant,
        ErrorVariant::NumericIndeterminate
    );
    assert_eq!(
        SInterval::<backend::Dashu>::between(-1.0, 1.0)?
            .ln()
            .unwrap_err()
            .variant,
        ErrorVariant::NumericDomain
    );
    Ok(())
}

#[test]
fn test_exp_subnormal_boundary() -> Fallible<()> {
    let minimum = f64::from_bits(1);
    let log_minimum = minimum.ln();

    for epsilon in [log_minimum.next_down(), log_minimum, log_minimum.next_up()] {
        let expected = high_precision_exp_bounds(epsilon)?;
        let interval = SInterval::<backend::Dashu>::point(epsilon)?.exp()?;
        assert_eq!(interval.lower_f64()?, expected.0);
        assert_eq!(interval.upper_f64()?, expected.1);
    }

    for x in [0.0, 1.0, 2.0] {
        let expected = high_precision_exp_bounds(x)?;
        let interval = SInterval::<backend::Dashu>::point(x)?.exp()?;
        assert_eq!(interval.lower_f64()?, expected.0);
        assert_eq!(interval.upper_f64()?, expected.1);
    }
    Ok(())
}
