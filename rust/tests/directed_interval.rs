//! Exercise generic intervals and their concrete aliases as a downstream crate.

use opendp::{
    error::Fallible,
    traits::{
        AInterval, Approximate, CInterval, Certified, DirectedScalar, DirectedTranscendental,
        Interval, N64, SInterval, SoftFloat,
    },
};

fn norm<S: DirectedScalar>(values: &[f64]) -> Fallible<Interval<S>> {
    values
        .iter()
        .try_fold(Interval::point(0.0)?, |sum, &x| {
            let x = Interval::point(x)?;
            sum + (x.clone() * x)?
        })?
        .sqrt()
}

fn check_generic_ordering<S: DirectedScalar>() -> Fallible<()> {
    let mut values = [
        S::exact(f64::INFINITY)?,
        -S::exact(2.0)?,
        S::exact(-0.0)?,
        S::exact(0.0)?,
    ];
    values.sort();
    assert!(values[1] == values[2]);
    let interval = Interval::new(values[0].clone(), values[3].clone())?;
    let contains_zero: bool = interval.contains_zero();
    assert!(contains_zero);
    assert!(!interval.is_nonnegative() && !interval.is_nonpositive());
    let magnitude: S = interval.clone().abs_upper();
    assert!(magnitude == S::exact(f64::INFINITY)?);
    let absolute: S = (-values[0].clone()).abs();
    assert!(absolute == S::exact(2.0)?);
    let negated: Interval<S> = -interval.clone();
    let minimum: Interval<S> = interval.clone().min(negated.clone());
    let maximum: Interval<S> = interval.max(negated);
    assert_eq!(minimum.lower_f64()?, f64::NEG_INFINITY);
    assert_eq!(minimum.upper_f64()?, 2.0);
    assert_eq!(maximum.lower_f64()?, -2.0);
    assert_eq!(maximum.upper_f64()?, f64::INFINITY);
    Ok(())
}

#[test]
fn generic_algorithms_use_standard_ordering_and_infallible_unary_operations() -> Fallible<()> {
    check_generic_ordering::<N64<Approximate>>()?;
    check_generic_ordering::<N64<Certified>>()?;
    check_generic_ordering::<SoftFloat>()
}

fn log<S: DirectedTranscendental>(x: Interval<S>) -> Fallible<Interval<S>> {
    x.ln()
}

fn check_generic_norm<S: DirectedScalar>() -> Fallible<()> {
    // Dyadic Pythagorean triples have an exact f64 norm. This gives a common
    // oracle even for approximate arithmetic, which makes no enclosure promise.
    for exponent in [-500, -100, -10, 0, 10, 100, 500] {
        let scale = 2.0f64.powi(exponent);
        for sign in [-1.0, 1.0] {
            for zeros in [0, 1, 16] {
                let mut values = vec![0.0; zeros];
                values.extend([sign * 3.0 * scale, -sign * 4.0 * scale]);
                let result = norm::<S>(&values)?;
                let expected = 5.0 * scale;
                assert!(result.lower_f64()? <= expected);
                assert!(result.upper_f64()? >= expected);
                let normalized = (result / Interval::point(expected)?)?.clamp01()?;
                assert!(normalized.lower_f64()? <= 1.0);
                assert_eq!(normalized.upper_f64()?, 1.0);
            }
        }
    }
    let zero = norm::<S>(&[0.0, -0.0])?;
    assert_eq!((zero.lower_f64()?, zero.upper_f64()?), (0.0, 0.0));
    Ok(())
}

#[test]
fn generic_algorithms_accept_scalars_directly() -> Fallible<()> {
    check_generic_norm::<N64<Approximate>>()?;
    check_generic_norm::<N64<Certified>>()?;
    check_generic_norm::<SoftFloat>()
}

#[test]
fn aliases_and_inferred_endpoints_are_interchangeable() -> Fallible<()> {
    let approximate: AInterval = norm::<N64<Approximate>>(&[3.0, 4.0])?;
    let certified: CInterval = norm::<N64<Certified>>(&[3.0, 4.0])?;
    let software: SInterval = norm::<SoftFloat>(&[3.0, 4.0])?;
    assert_eq!(approximate.lower_f64()?, 5.0);
    assert!(certified.upper_f64()? >= 5.0);
    assert_eq!(software.upper_f64()?, 5.0);

    // No explicit Interval type parameter is needed.
    let inferred = Interval::new(N64::<Certified>::exact(0.0)?, N64::exact(1.0)?)?;
    let _: &N64<Certified> = inferred.lower();
    let _: &N64<Certified> = inferred.upper();
    let (lo, hi) = inferred.into_endpoints();
    let rebuilt: CInterval = Interval::new(lo, hi)?;
    assert_eq!(rebuilt.sqrt()?.upper_f64()?, 1.0);
    assert_eq!(log(SInterval::point(1.0)?)?.lower_f64()?, 0.0);
    Ok(())
}
