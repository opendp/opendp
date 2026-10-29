use super::*;
use crate::error::ErrorVariant;
use dashu::{
    float::{
        FBig,
        round::mode::{Down, Up},
    },
    rational::RBig,
};

fn assert_contains<S: DirectedScalar>(
    interval: &Interval<S>,
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

fn check_arithmetic<S: DirectedScalar>() -> Fallible<()> {
    assert_contains(
        &(Interval::<S>::between(1.0, 2.0)? + Interval::between(3.0, 4.0)?)?,
        4.0,
        6.0,
    )?;
    assert_contains(
        &(Interval::<S>::between(1.0, 2.0)? - Interval::between(3.0, 4.0)?)?,
        -3.0,
        -1.0,
    )?;
    assert_contains(&-Interval::<S>::between(1.0, 2.0)?, -2.0, -1.0)?;

    for (lhs, rhs, expected) in [
        ((1.0, 2.0), (3.0, 4.0), (3.0, 8.0)),
        ((-2.0, -1.0), (-4.0, -3.0), (3.0, 8.0)),
        ((1.0, 2.0), (-4.0, -3.0), (-8.0, -3.0)),
        ((-2.0, -1.0), (3.0, 4.0), (-8.0, -3.0)),
        ((-2.0, 3.0), (-4.0, 5.0), (-12.0, 15.0)),
    ] {
        let product = (Interval::<S>::between(lhs.0, lhs.1)? * Interval::between(rhs.0, rhs.1)?)?;
        assert_contains(&product, expected.0, expected.1)?;
    }

    assert_contains(
        &(Interval::<S>::between(2.0, 4.0)? / Interval::between(1.0, 2.0)?)?,
        1.0,
        4.0,
    )?;
    assert_contains(&Interval::<S>::between(-3.0, 2.0)?.abs()?, 0.0, 3.0)?;
    assert!(Interval::<S>::between(-1.0, 1.0)?.recip().is_err());
    Ok(())
}

#[test]
fn test_interval_arithmetic_backends() -> Fallible<()> {
    check_arithmetic::<N64<Approximate>>()?;
    check_arithmetic::<N64<Certified>>()?;
    check_arithmetic::<SoftFloat>()
}

fn check_direct_division<S: DirectedScalar + std::fmt::Debug>() -> Fallible<()> {
    let tiny = f64::from_bits(1);
    let inf = f64::INFINITY;
    for (lhs, rhs, expected) in [
        ((0.0, 0.0), (tiny, tiny), (0.0, 0.0)),
        ((tiny, tiny), (tiny, tiny), (1.0, 1.0)),
        ((-tiny, -tiny), (tiny, tiny), (-1.0, -1.0)),
        ((tiny, tiny), (-tiny, -tiny), (-1.0, -1.0)),
        ((tiny, 2.0 * tiny), (tiny, 2.0 * tiny), (0.5, 2.0)),
        ((-2.0 * tiny, -tiny), (tiny, 2.0 * tiny), (-2.0, -0.5)),
        ((-2.0 * tiny, -tiny), (-2.0 * tiny, -tiny), (0.5, 2.0)),
        ((-tiny, tiny), (tiny, 2.0 * tiny), (-1.0, 1.0)),
        ((-tiny, tiny), (-2.0 * tiny, -tiny), (-1.0, 1.0)),
        ((1.0, inf), (1.0, inf), (0.0, inf)),
        ((-inf, -1.0), (1.0, inf), (-inf, 0.0)),
        ((-inf, inf), (1.0, inf), (-inf, inf)),
        ((1.0, inf), (-inf, -1.0), (-inf, 0.0)),
        ((-inf, -1.0), (-inf, -1.0), (0.0, inf)),
        ((-inf, inf), (-inf, -1.0), (-inf, inf)),
    ] {
        let result = (Interval::<S>::between(lhs.0, lhs.1)? / Interval::between(rhs.0, rhs.1)?)?;
        assert_contains(&result, expected.0, expected.1)?;
        let actual = (result.lower_f64()?, result.upper_f64()?);
        if lhs.0 == lhs.1 && rhs.0 == rhs.1 {
            // Zero and identical-subnormal quotients must stay exact.
            assert_eq!(actual, expected, "{lhs:?} / {rhs:?}");
        } else {
            // Certified native arithmetic may widen exact dyadic results by one
            // outward ULP, but must not inherit reciprocal overflow/widening.
            assert!(
                actual.0 >= expected.0.next_down(),
                "{lhs:?} / {rhs:?}: {actual:?}"
            );
            assert!(
                actual.1 <= expected.1.next_up(),
                "{lhs:?} / {rhs:?}: {actual:?}"
            );
        }
    }
    for (numerator, divisor) in [
        (0.0, 2.0),
        (-0.0, 2.0),
        (0.0, -2.0),
        (-0.0, -2.0),
        (1.0, inf),
        (-1.0, inf),
        (1.0, -inf),
        (-1.0, -inf),
    ] {
        let result = (Interval::<S>::point(numerator)? / Interval::point(divisor)?)?;
        let expected = (numerator / divisor).to_bits();
        assert_eq!(result.lower_f64()?.to_bits(), expected);
        assert_eq!(result.upper_f64()?.to_bits(), expected);
    }
    for (lo, hi) in [(0.0, 0.0), (-0.0, 0.0), (-1.0, 1.0)] {
        assert_eq!(
            (Interval::<S>::point(1.0)? / Interval::between(lo, hi)?)
                .unwrap_err()
                .variant,
            ErrorVariant::NumericIndeterminate,
        );
    }
    Ok(())
}

#[test]
fn test_direct_division_backends() -> Fallible<()> {
    check_direct_division::<N64<Approximate>>()?;
    check_direct_division::<N64<Certified>>()?;
    check_direct_division::<SoftFloat>()
}

fn check_native_finite_overflow<S: DirectedScalar>() -> Fallible<()> {
    let positive = (Interval::<S>::point(f64::MAX)? + Interval::point(f64::MAX)?)?;
    assert_eq!(positive.lower_f64()?, f64::MAX);
    assert_eq!(positive.upper_f64()?, f64::INFINITY);

    let negative = (Interval::<S>::point(-f64::MAX)? + Interval::point(-f64::MAX)?)?;
    assert_eq!(negative.lower_f64()?, f64::NEG_INFINITY);
    assert_eq!(negative.upper_f64()?, -f64::MAX);
    Ok(())
}

#[test]
fn test_native_finite_overflow_is_directionally_enclosed() -> Fallible<()> {
    check_native_finite_overflow::<N64<Certified>>()?;

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
    let positive_infinity = SInterval::point(f64::INFINITY)?;
    let negative_infinity = SInterval::point(f64::NEG_INFINITY)?;
    let zero = SInterval::point(0.0)?;

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
        SInterval::point(-1.0)?.ln().unwrap_err().variant,
        crate::error::ErrorVariant::NumericDomain
    );
    assert_eq!(
        SInterval::point(-1.0)?.sqrt().unwrap_err().variant,
        crate::error::ErrorVariant::NumericDomain
    );
    Ok(())
}

#[test]
fn test_interval_construction_and_clamping() -> Fallible<()> {
    assert!(CInterval::between(2.0, 1.0).is_err());
    assert!(CInterval::point(f64::NAN).is_err());
    assert_eq!(CInterval::point(f64::INFINITY)?.lower_f64()?, f64::INFINITY);
    assert!(CInterval::between(-1.0, 2.0)?.contains_zero());
    assert!(CInterval::between(0.0, 2.0)?.is_nonnegative());
    assert!(CInterval::between(-2.0, 0.0)?.is_nonpositive());

    assert_contains(&CInterval::between(-2.0, 3.0)?.clamp01()?, 0.0, 1.0)?;
    assert!(CInterval::point(0.5)?.clamp(1.0, 0.0).is_err());

    assert_contains(
        &CInterval::between(1.0, 3.0)?.min(CInterval::between(2.0, 4.0)?),
        1.0,
        3.0,
    )?;
    assert_contains(
        &CInterval::between(1.0, 3.0)?.max(CInterval::between(2.0, 4.0)?),
        2.0,
        4.0,
    )?;
    assert_eq!(
        CInterval::between(-2.0, 3.0)?
            .abs_upper()
            .to_f64(Direction::Up)?,
        3.0
    );
    Ok(())
}

fn check_interval_ordering<S: DirectedScalar + std::fmt::Debug>() -> Fallible<()> {
    for (lo, hi) in [
        (f64::NEG_INFINITY, f64::INFINITY),
        (f64::NEG_INFINITY, -1.0),
        (f64::NEG_INFINITY, f64::NEG_INFINITY),
        (f64::INFINITY, f64::INFINITY),
        (-f64::from_bits(1), f64::from_bits(1)),
        (f64::from_bits(1), f64::INFINITY),
        (0.0, 1.0),
        (-1.0, -0.0),
        (-0.0, 0.0),
        (0.0, -0.0),
    ] {
        let interval = Interval::<S>::new(S::exact(lo)?, S::exact(hi)?)?;
        assert_eq!(interval.lower_f64()?.to_bits(), lo.to_bits());
        assert_eq!(interval.upper_f64()?.to_bits(), hi.to_bits());
        assert_eq!(interval.contains_zero(), lo <= 0.0 && hi >= 0.0);
        assert_eq!(interval.is_nonnegative(), lo >= 0.0);
        assert_eq!(interval.is_nonpositive(), hi <= 0.0);
        assert_eq!(
            interval.abs_upper().to_f64(Direction::Up)?,
            lo.abs().max(hi.abs())
        );
    }
    for (lo, hi) in [
        (1.0, 0.0),
        (f64::INFINITY, f64::MAX),
        (-f64::MAX, f64::NEG_INFINITY),
    ] {
        assert_eq!(
            Interval::<S>::between(lo, hi).unwrap_err().variant,
            ErrorVariant::FailedFunction
        );
    }
    for (lo, hi) in [(f64::NAN, 0.0), (0.0, f64::NAN)] {
        assert_eq!(
            Interval::<S>::between(lo, hi).unwrap_err().variant,
            ErrorVariant::NumericIndeterminate
        );
    }
    // Standard Ord extrema select the first minimum and last maximum on ties.
    for (lhs, rhs) in [(-0.0f64, 0.0f64), (0.0f64, -0.0f64)] {
        let left = Interval::<S>::point(lhs)?;
        let right = Interval::<S>::point(rhs)?;
        let minimum = left.clone().min(right.clone());
        let maximum = left.max(right);
        assert_eq!(minimum.lower_f64()?.to_bits(), lhs.to_bits());
        assert_eq!(minimum.upper_f64()?.to_bits(), lhs.to_bits());
        assert_eq!(maximum.lower_f64()?.to_bits(), rhs.to_bits());
        assert_eq!(maximum.upper_f64()?.to_bits(), rhs.to_bits());
    }
    Ok(())
}

#[test]
fn test_interval_ordering_and_predicates() -> Fallible<()> {
    check_interval_ordering::<N64<Approximate>>()?;
    check_interval_ordering::<N64<Certified>>()?;
    check_interval_ordering::<SoftFloat>()
}

#[test]
fn test_internal_endpoint_ordering() -> Fallible<()> {
    let small = SoftFloat::exact(-10_000.0)?.exp_round(Direction::Up)?;
    assert_eq!(small.to_f64(Direction::Down)?, 0.0);
    let positive = Interval::new(small.clone(), small.clone())?;
    assert!(!positive.contains_zero() && positive.is_nonnegative() && !positive.is_nonpositive());
    assert!(positive.recip().is_ok());
    let negative = Interval::new(-small.clone(), -small.clone())?;
    assert!(!negative.contains_zero() && !negative.is_nonnegative() && negative.is_nonpositive());
    assert_eq!(
        negative.sqrt().unwrap_err().variant,
        ErrorVariant::NumericDomain
    );
    assert_eq!(
        Interval::new(small, SoftFloat::exact(0.0)?)
            .unwrap_err()
            .variant,
        ErrorVariant::FailedFunction
    );
    let one = SoftFloat::exact(1.0)?;
    let above = one
        .clone()
        .add_round(SoftFloat::exact(2.0f64.powi(-255))?, Direction::Up)?;
    assert_eq!(above.to_f64(Direction::Down)?, 1.0);
    let interval = Interval::new(one.clone(), above.clone())?;
    assert_eq!(
        format!("{interval:?}"),
        format!("Interval {{ lo: {one:?}, hi: {above:?} }}")
    );
    assert_eq!(
        Interval::new(above, one).unwrap_err().variant,
        ErrorVariant::FailedFunction
    );
    Ok(())
}

fn check_transcendentals<S: DirectedTranscendental + std::fmt::Debug>() -> Fallible<()> {
    assert_contains(
        &Interval::<S>::point(1.0)?.exp()?,
        std::f64::consts::E,
        std::f64::consts::E,
    )?;
    assert_contains(
        &Interval::<S>::point(1.0)?.exp_m1()?,
        std::f64::consts::E - 1.0,
        1.0_f64.exp_m1(),
    )?;
    assert_contains(&Interval::<S>::point(1.0)?.ln()?, 0.0, 0.0)?;
    assert_eq!(
        Interval::<S>::between(-1.0, 1.0)?.ln().unwrap_err().variant,
        ErrorVariant::NumericDomain
    );

    let boundary = Interval::<S>::between(0.0, 1.0)?.ln()?;
    assert_eq!(boundary.lower_f64()?, f64::NEG_INFINITY);
    assert!(boundary.upper_f64()? >= 0.0);
    Ok(())
}

#[test]
fn test_interval_transcendental_backends() -> Fallible<()> {
    let approximate = AInterval::point(1.0)?.exp_m1()?;
    assert_eq!(approximate.lower_f64()?, 1.0_f64.exp_m1());
    assert_eq!(approximate.upper_f64()?, 1.0_f64.exp_m1());

    check_transcendentals::<SoftFloat>()?;
    Ok(())
}

fn check_sqrt<S: DirectedScalar + std::fmt::Debug>() -> Fallible<()> {
    assert_contains(&Interval::<S>::between(0.0, 4.0)?.sqrt()?, 0.0, 2.0)?;
    assert_eq!(
        Interval::<S>::between(-1.0, 1.0)?
            .sqrt()
            .unwrap_err()
            .variant,
        ErrorVariant::NumericDomain
    );
    for x in [0.0, 1.0, f64::INFINITY] {
        let result = Interval::<S>::point(x)?.sqrt()?;
        assert_eq!(result.lower_f64()?, x);
        assert_eq!(result.upper_f64()?, x);
    }
    Ok(())
}

#[test]
fn test_interval_sqrt_backends() -> Fallible<()> {
    check_sqrt::<N64<Approximate>>()?;
    check_sqrt::<N64<Certified>>()?;
    check_sqrt::<SoftFloat>()
}

#[test]
fn test_certified_identity_sqrt_interactions() -> Fallible<()> {
    // Exact zero must not widen below the square-root domain.
    for zero in [
        (CInterval::point(3.0)? - CInterval::point(3.0)?)?,
        (CInterval::between(-2.0, 3.0)? * CInterval::point(0.0)?)?,
        (CInterval::point(0.0)? / CInterval::point(2.0)?)?,
    ] {
        let root = zero.sqrt()?;
        assert_eq!(root.lower_f64()?, 0.0);
        assert_eq!(root.upper_f64()?, 0.0);
    }
    for interval in [
        (CInterval::between(0.0, 4.0)? + CInterval::point(0.0)?)?,
        (CInterval::between(0.0, 4.0)? * CInterval::point(1.0)?)?,
        (CInterval::between(0.0, 4.0)? / CInterval::point(1.0)?)?,
    ] {
        assert_eq!(interval.lower_f64()?, 0.0);
        assert_eq!(interval.upper_f64()?, 4.0);
        assert_contains(&interval.sqrt()?, 0.0, 2.0)?;
    }
    let signed_zero = CInterval::point(-0.0)?.sqrt()?;
    assert_eq!(signed_zero.lower_f64()?.to_bits(), (-0.0f64).to_bits());
    assert_eq!(signed_zero.upper_f64()?.to_bits(), (-0.0f64).to_bits());
    Ok(())
}

#[test]
fn test_positive_underflow_sqrt() -> Fallible<()> {
    let minimum = f64::from_bits(1);
    for factor in [0.5, 0.25] {
        let product = (CInterval::point(minimum)? * CInterval::point(factor)?)?;
        assert_eq!(product.lower_f64()?.to_bits(), 0.0f64.to_bits());
        assert_eq!(product.upper_f64()?, minimum);
        // The exact product is strictly positive, although its native value
        // rounds to zero. Its lower bound must not spuriously reject sqrt.
        let root = product.sqrt()?;
        let lo = RBig::try_from(root.lower_f64()?)?;
        let hi = RBig::try_from(root.upper_f64()?)?;
        let exact_product = RBig::try_from(minimum)? * RBig::try_from(factor)?;
        assert!(lo >= RBig::ZERO);
        assert!(&lo * &lo <= exact_product);
        assert!(&hi * &hi >= exact_product);
    }
    Ok(())
}

fn high_precision_exp_bounds(x: f64) -> Fallible<(RBig, RBig)> {
    let lower = FBig::<Down>::try_from(x)?
        .with_precision(1024)
        .value()
        .exp();
    let upper = FBig::<Up>::try_from(x)?.with_precision(1024).value().exp();
    Ok((RBig::try_from(lower)?, RBig::try_from(upper)?))
}

#[test]
fn test_interval_error_propagation() -> Fallible<()> {
    let positive = SInterval::point(10_000.0)?.exp()?;
    assert_eq!(
        positive.upper_f64().unwrap_err().variant,
        ErrorVariant::NumericRangeAbove
    );

    let negative = -positive;
    assert_eq!(
        negative.lower_f64().unwrap_err().variant,
        ErrorVariant::NumericRangeBelow
    );

    assert_eq!(
        SInterval::between(-1.0, 1.0)?.recip().unwrap_err().variant,
        ErrorVariant::NumericIndeterminate
    );
    assert_eq!(
        SInterval::between(-1.0, 1.0)?.ln().unwrap_err().variant,
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
        let interval = SInterval::point(epsilon)?.exp()?;
        // Compare against the exact rational high-precision endpoints, not
        // rounded f64 bounds with an inward tolerance.
        assert!(RBig::try_from(interval.lower_f64()?)? <= expected.0);
        assert!(RBig::try_from(interval.upper_f64()?)? >= expected.1);
    }

    for x in [0.0, 1.0, 2.0] {
        let expected = high_precision_exp_bounds(x)?;
        let interval = SInterval::point(x)?.exp()?;
        assert!(RBig::try_from(interval.lower_f64()?)? <= expected.0);
        assert!(RBig::try_from(interval.upper_f64()?)? >= expected.1);
    }
    Ok(())
}
