use super::*;
use crate::{
    error::{ErrorVariant, Fallible},
    traits::directed::backend::Dashu,
};
use dashu::rational::RBig;

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
    check_native_overflow::<BestEffort>()?;
    check_native_overflow::<Certified>()?;

    // Approximate arithmetic returns the native result without outward
    // widening, including native infinities caused by finite overflow.
    let max = N64::<Approximate>::exact(f64::MAX)?;
    for direction in [Direction::Down, Direction::Up] {
        assert_eq!(
            max.clone()
                .add_round(max.clone(), direction)?
                .to_f64(direction)?,
            f64::INFINITY
        );
        assert_eq!(
            N64::<Approximate>::exact(-f64::MAX)?
                .add_round(N64::exact(-f64::MAX)?, direction)?
                .to_f64(direction)?,
            f64::NEG_INFINITY
        );
    }
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
    for direction in [Direction::Down, Direction::Up] {
        assert_eq!(
            N64::<BestEffort>::approx(f64::INFINITY, direction)?.to_f64(direction)?,
            f64::INFINITY
        );
        assert_eq!(
            N64::<Certified>::approx(f64::NEG_INFINITY, direction)?.to_f64(direction)?,
            f64::NEG_INFINITY
        );
    }
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

fn check_native_transcendental_identities<R: NativeTranscendental>() -> Fallible<()> {
    for direction in [Direction::Down, Direction::Up] {
        assert_eq!(
            N64::<R>::exact(f64::NEG_INFINITY)?
                .exp_round(direction)?
                .to_f64(direction)?,
            0.0
        );
        assert_eq!(
            N64::<R>::exact(f64::INFINITY)?
                .exp_round(direction)?
                .to_f64(direction)?,
            f64::INFINITY
        );
        assert_eq!(
            N64::<R>::exact(0.0)?
                .exp_round(direction)?
                .to_f64(direction)?,
            1.0
        );

        assert_eq!(
            N64::<R>::exact(f64::NEG_INFINITY)?
                .exp_m1_round(direction)?
                .to_f64(direction)?,
            -1.0
        );
        assert_eq!(
            N64::<R>::exact(f64::INFINITY)?
                .exp_m1_round(direction)?
                .to_f64(direction)?,
            f64::INFINITY
        );
        assert_eq!(
            N64::<R>::exact(0.0)?
                .exp_m1_round(direction)?
                .to_f64(direction)?,
            0.0
        );

        assert_eq!(
            N64::<R>::exact(0.0)?
                .ln_round(direction)?
                .to_f64(direction)?,
            f64::NEG_INFINITY
        );
        assert_eq!(
            N64::<R>::exact(1.0)?
                .ln_round(direction)?
                .to_f64(direction)?,
            0.0
        );
        assert_eq!(
            N64::<R>::exact(f64::INFINITY)?
                .ln_round(direction)?
                .to_f64(direction)?,
            f64::INFINITY
        );

        assert_eq!(
            N64::<R>::exact(0.0)?
                .sqrt_round(direction)?
                .to_f64(direction)?,
            0.0
        );
        assert_eq!(
            N64::<R>::exact(1.0)?
                .sqrt_round(direction)?
                .to_f64(direction)?,
            1.0
        );
        assert_eq!(
            N64::<R>::exact(f64::INFINITY)?
                .sqrt_round(direction)?
                .to_f64(direction)?,
            f64::INFINITY
        );
    }
    Ok(())
}

#[test]
fn test_native_transcendental_identities() -> Fallible<()> {
    check_native_transcendental_identities::<Approximate>()?;
    check_native_transcendental_identities::<BestEffort>()?;
    Ok(())
}

#[test]
fn test_dashu_special_values_and_range_errors() -> Fallible<()> {
    assert_error(D::exact(f64::NAN), ErrorVariant::NumericIndeterminate);
    assert_eq!(
        D::exact(f64::INFINITY)?.to_f64(Direction::Down)?,
        f64::INFINITY
    );
    assert_eq!(
        D::exact(f64::INFINITY)?.to_f64(Direction::Up)?,
        f64::INFINITY
    );
    assert_eq!(
        D::exact(f64::NEG_INFINITY)?.to_f64(Direction::Down)?,
        f64::NEG_INFINITY
    );
    assert_eq!(
        D::exact(f64::NEG_INFINITY)?.to_f64(Direction::Up)?,
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
    assert_eq!(positive.to_f64(Direction::Down)?, f64::MAX);
    assert_error(
        positive.to_f64(Direction::Up),
        ErrorVariant::NumericRangeAbove,
    );
    let negative = positive.neg()?;
    assert_error(
        negative.to_f64(Direction::Down),
        ErrorVariant::NumericRangeBelow,
    );
    assert_eq!(negative.to_f64(Direction::Up)?, -f64::MAX);
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

#[derive(Clone, Copy)]
enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
}

const ENCLOSURE_CASES: &[(f64, f64)] = &[
    (0.1, 0.2),
    (1.0, 3.0),
    (1e16, -1e16),
    (-0.1, 0.2),
    (1e-300, -1e-300),
    (f64::MIN_POSITIVE, f64::from_bits(1)),
];

fn rational(value: f64) -> RBig {
    RBig::try_from(value).expect("finite f64 is an exact rational")
}

fn expected(lhs: f64, rhs: f64, operation: BinaryOperation) -> RBig {
    let lhs = rational(lhs);
    let rhs = rational(rhs);
    match operation {
        BinaryOperation::Add => lhs + rhs,
        BinaryOperation::Sub => lhs - rhs,
        BinaryOperation::Mul => lhs * rhs,
        BinaryOperation::Div => lhs / rhs,
    }
}

fn assert_encloses(down: f64, up: f64, expected: RBig) {
    assert!(
        rational(down) <= expected,
        "downward result {down:?} is above exact result {expected:?}"
    );
    assert!(
        expected <= rational(up),
        "upward result {up:?} is below exact result {expected:?}"
    );
}

fn native_result<R: NativeRegime>(
    lhs: f64,
    rhs: f64,
    operation: BinaryOperation,
    direction: Direction,
) -> Fallible<N64<R>> {
    let lhs = N64::<R>::exact(lhs)?;
    let rhs = N64::<R>::exact(rhs)?;
    match operation {
        BinaryOperation::Add => lhs.add_round(rhs, direction),
        BinaryOperation::Sub => lhs.sub_round(rhs, direction),
        BinaryOperation::Mul => lhs.mul_round(rhs, direction),
        BinaryOperation::Div => lhs.div_round(rhs, direction),
    }
}

fn dashu_result(
    lhs: f64,
    rhs: f64,
    operation: BinaryOperation,
    direction: Direction,
) -> Fallible<D> {
    let lhs = D::exact(lhs)?;
    let rhs = D::exact(rhs)?;
    match operation {
        BinaryOperation::Add => lhs.add_round(rhs, direction),
        BinaryOperation::Sub => lhs.sub_round(rhs, direction),
        BinaryOperation::Mul => lhs.mul_round(rhs, direction),
        BinaryOperation::Div => lhs.div_round(rhs, direction),
    }
}

fn check_native_enclosures<R: NativeRegime>() -> Fallible<()> {
    for &(lhs, rhs) in ENCLOSURE_CASES {
        for operation in [
            BinaryOperation::Add,
            BinaryOperation::Sub,
            BinaryOperation::Mul,
            BinaryOperation::Div,
        ] {
            let down = native_result::<R>(lhs, rhs, operation, Direction::Down)?
                .to_f64(Direction::Down)?;
            let up =
                native_result::<R>(lhs, rhs, operation, Direction::Up)?.to_f64(Direction::Up)?;
            assert_encloses(down, up, expected(lhs, rhs, operation));
        }
    }
    Ok(())
}

fn check_dashu_enclosures() -> Fallible<()> {
    for &(lhs, rhs) in ENCLOSURE_CASES {
        for operation in [
            BinaryOperation::Add,
            BinaryOperation::Sub,
            BinaryOperation::Mul,
            BinaryOperation::Div,
        ] {
            let down =
                dashu_result(lhs, rhs, operation, Direction::Down)?.to_f64(Direction::Down)?;
            let up = dashu_result(lhs, rhs, operation, Direction::Up)?.to_f64(Direction::Up)?;
            assert_encloses(down, up, expected(lhs, rhs, operation));
        }
    }
    Ok(())
}

#[test]
fn test_representative_arithmetic_enclosures() -> Fallible<()> {
    check_native_enclosures::<BestEffort>()?;
    check_native_enclosures::<Certified>()?;
    check_dashu_enclosures()?;
    Ok(())
}
