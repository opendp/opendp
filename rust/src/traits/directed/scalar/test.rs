use super::*;
use crate::{
    error::{ErrorVariant, Fallible},
    traits::directed::backend::Dashu,
};

type D = SoftFloat<Dashu>;

fn assert_error<T: std::fmt::Debug>(result: Fallible<T>, variant: ErrorVariant) {
    assert_eq!(result.unwrap_err().variant, variant);
}

fn check_native_overflow<R: NativeRegime>() -> Fallible<()> {
    let max = N64::<R>::exact(f64::MAX)?;
    let two = N64::<R>::exact(2.0)?;
    let half = N64::<R>::exact(0.5)?;
    let negative_max = N64::<R>::exact(-f64::MAX)?;

    assert_eq!(
        max.clone()
            .add_round(max.clone(), Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::MAX
    );
    assert_eq!(
        max.clone()
            .add_round(max, Direction::Up)?
            .to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .add_round(N64::exact(-f64::MAX)?, Direction::Down,)?
            .to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .add_round(N64::exact(-f64::MAX)?, Direction::Up,)?
            .to_f64(Direction::Up)?,
        -f64::MAX
    );
    assert_eq!(
        max.clone()
            .sub_round(N64::exact(-f64::MAX)?, Direction::Down,)?
            .to_f64(Direction::Down)?,
        f64::MAX
    );
    assert_eq!(
        max.clone()
            .sub_round(N64::exact(-f64::MAX)?, Direction::Up,)?
            .to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .sub_round(N64::exact(f64::MAX)?, Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .sub_round(N64::exact(f64::MAX)?, Direction::Up)?
            .to_f64(Direction::Up)?,
        -f64::MAX
    );
    assert_eq!(
        negative_max
            .clone()
            .mul_round(two.clone(), Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .mul_round(two.clone(), Direction::Up)?
            .to_f64(Direction::Up)?,
        -f64::MAX
    );
    assert_eq!(
        max.clone()
            .mul_round(two.clone(), Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::MAX
    );
    assert_eq!(
        max.clone()
            .mul_round(two, Direction::Up)?
            .to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        max.clone()
            .div_round(half.clone(), Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::MAX
    );
    assert_eq!(
        max.div_round(half, Direction::Up)?.to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        negative_max
            .clone()
            .div_round(N64::exact(0.5)?, Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_eq!(
        negative_max
            .div_round(N64::exact(0.5)?, Direction::Up)?
            .to_f64(Direction::Up)?,
        -f64::MAX
    );
    Ok(())
}

#[test]
fn test_native_overflow_all_regimes() -> Fallible<()> {
    check_native_overflow::<Approximate>()?;
    check_native_overflow::<BestEffort>()?;
    check_native_overflow::<Certified>()?;
    Ok(())
}

#[test]
fn test_native_special_values_and_domains() -> Fallible<()> {
    assert_error(
        N64::<Certified>::exact(f64::NAN),
        ErrorVariant::NumericIndeterminate,
    );
    assert_eq!(
        N64::<Certified>::exact(f64::INFINITY)?.to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        N64::<Certified>::exact(f64::NEG_INFINITY)?.to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_error(
        N64::<Certified>::exact(f64::INFINITY)?
            .add_round(N64::exact(f64::NEG_INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        N64::<Certified>::exact(0.0)?.mul_round(N64::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        N64::<Certified>::exact(f64::INFINITY)?
            .div_round(N64::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        N64::<Approximate>::exact(-1.0)?.ln_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );
    assert_error(
        N64::<Approximate>::exact(-1.0)?.sqrt_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );
    assert_eq!(
        N64::<Approximate>::exact(0.0)?
            .ln_round(Direction::Down)?
            .to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    Ok(())
}

#[test]
fn test_dashu_special_values_and_range_errors() -> Fallible<()> {
    assert_error(D::exact(f64::NAN), ErrorVariant::NumericIndeterminate);
    assert_eq!(
        D::exact(f64::INFINITY)?.to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        D::exact(f64::NEG_INFINITY)?.to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_error(
        D::exact(f64::INFINITY)?.add_round(D::exact(f64::NEG_INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        D::exact(0.0)?.mul_round(D::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        D::exact(f64::INFINITY)?.div_round(D::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        D::exact(-1.0)?.ln_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );
    assert_error(
        D::exact(-1.0)?.sqrt_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );

    let positive = D::exact(10000.0)?.exp_round(Direction::Up)?;
    assert_error(
        positive.to_f64(Direction::Up),
        ErrorVariant::NumericRangeAbove,
    );
    let negative = positive.neg()?;
    assert_error(
        negative.to_f64(Direction::Down),
        ErrorVariant::NumericRangeBelow,
    );
    Ok(())
}

#[test]
fn test_dashu_subnormal_conversion() -> Fallible<()> {
    let positive = D::exact(-10000.0)?.exp_round(Direction::Up)?;
    assert_eq!(positive.to_f64(Direction::Down)?, 0.0);
    assert_eq!(positive.to_f64(Direction::Up)?, f64::from_bits(1));

    let negative = positive.neg()?;
    assert_eq!(negative.to_f64(Direction::Down)?, -f64::from_bits(1));
    assert_eq!(negative.to_f64(Direction::Up)?, -0.0);
    Ok(())
}

#[test]
fn test_consuming_arithmetic_and_comparison() -> Fallible<()> {
    let lhs = D::exact(2.0)?;
    let rhs = D::exact(3.0)?;
    let sum = lhs.add_round(rhs, Direction::Down)?;
    assert_eq!(sum.compare(&D::exact(5.0)?)?, std::cmp::Ordering::Equal);
    Ok(())
}
