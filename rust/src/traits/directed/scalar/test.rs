use super::*;
use crate::{
    error::{ErrorVariant, Fallible},
    traits::directed::backend::{Dashu, Rug},
};
use dashu::rational::RBig;

type D = SoftFloat<Dashu>;
type R = SoftFloat<Rug>;

fn assert_error<T>(result: Fallible<T>, variant: ErrorVariant) {
    match result {
        Ok(_) => panic!("expected {variant:?}"),
        Err(error) => assert_eq!(error.variant, variant),
    }
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
    // Division by zero is intentionally classified as indeterminate: the
    // directed scalar API does not assign an extended-real result to it.
    assert_error(
        N64::<Certified>::exact(1.0)?.div_round(N64::exact(0.0)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        D::exact(1.0)?.div_round(D::exact(-0.0)?, Direction::Up),
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

fn check_software_special_values<S: DirectedTranscendental>() -> Fallible<()> {
    assert_error(S::exact(f64::NAN), ErrorVariant::NumericIndeterminate);
    for direction in [Direction::Down, Direction::Up] {
        assert_eq!(S::exact(f64::INFINITY)?.to_f64(direction)?, f64::INFINITY);
        assert_eq!(
            S::exact(f64::NEG_INFINITY)?.to_f64(direction)?,
            f64::NEG_INFINITY
        );
    }
    assert_error(
        S::exact(f64::INFINITY)?.add_round(S::exact(f64::NEG_INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        S::exact(0.0)?.mul_round(S::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        S::exact(f64::INFINITY)?.div_round(S::exact(f64::INFINITY)?, Direction::Down),
        ErrorVariant::NumericIndeterminate,
    );
    assert_error(
        S::exact(-1.0)?.ln_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );
    assert_error(
        S::exact(-1.0)?.sqrt_round(Direction::Down),
        ErrorVariant::NumericDomain,
    );

    let positive = S::exact(10000.0)?.exp_round(Direction::Up)?;
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
fn test_software_special_values_and_range_errors() -> Fallible<()> {
    check_software_special_values::<D>()?;
    check_software_special_values::<R>()
}

fn check_software_subnormal_conversion<S: DirectedTranscendental>() -> Fallible<()> {
    let positive = S::exact(-10000.0)?.exp_round(Direction::Up)?;
    assert_eq!(positive.to_f64(Direction::Down)?, 0.0);
    assert_eq!(positive.to_f64(Direction::Up)?, f64::from_bits(1));

    let negative = positive.neg()?;
    assert_eq!(negative.to_f64(Direction::Down)?, -f64::from_bits(1));
    assert_eq!(negative.to_f64(Direction::Up)?, -0.0);
    Ok(())
}

#[test]
fn test_software_subnormal_conversion() -> Fallible<()> {
    check_software_subnormal_conversion::<D>()?;
    check_software_subnormal_conversion::<R>()
}

fn check_consuming_arithmetic_and_comparison<S: DirectedScalar>() -> Fallible<()> {
    let lhs = S::exact(2.0)?;
    let rhs = S::exact(3.0)?;
    let sum = lhs.add_round(rhs, Direction::Down)?;
    assert_eq!(sum.compare(&S::exact(5.0)?)?, std::cmp::Ordering::Equal);
    Ok(())
}

#[test]
fn test_consuming_arithmetic_and_comparison() -> Fallible<()> {
    check_consuming_arithmetic_and_comparison::<D>()?;
    check_consuming_arithmetic_and_comparison::<R>()
}

#[derive(Clone, Copy)]
enum BinaryOperation {
    Add,
    Sub,
    Mul,
    Div,
}

fn enclosure_values() -> Vec<f64> {
    let mut values = vec![
        0.0,
        -0.0,
        f64::from_bits(1),
        -f64::from_bits(1),
        f64::from_bits(2),
        -f64::from_bits(2),
        f64::MIN_POSITIVE,
        -f64::MIN_POSITIVE,
        0.5,
        -0.5,
        1.0,
        -1.0,
        2.0,
        -2.0,
        1e-300,
        -1e-300,
        1e300,
        -1e300,
        f64::MAX / 2.0,
        -f64::MAX / 2.0,
        f64::MAX,
        -f64::MAX,
    ];

    // Add adjacent values around powers of two and one. This is deterministic
    // while exercising both sides of normal/subnormal and rounding boundaries.
    for value in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        2f64.powi(-52),
        1.0,
        2.0,
        2f64.powi(52),
        2f64.powi(53),
        2f64.powi(100),
        f64::MAX / 2.0,
        f64::MAX,
    ] {
        for candidate in [
            value.next_down(),
            value,
            value.next_up(),
            (-value).next_down(),
            -value,
            (-value).next_up(),
        ] {
            if candidate.is_finite() {
                values.push(candidate);
            }
        }
    }
    values
}

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
    // An outward result may be an infinity when the exact result is at an
    // f64 boundary; infinity is ordered beyond every finite exact rational.
    if down != f64::NEG_INFINITY {
        assert!(
            rational(down) <= expected,
            "downward result {down:?} is above exact result {expected:?}"
        );
    }
    if up != f64::INFINITY {
        assert!(
            expected <= rational(up),
            "upward result {up:?} is below exact result {expected:?}"
        );
    }
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

fn software_result<S: DirectedScalar>(
    lhs: f64,
    rhs: f64,
    operation: BinaryOperation,
    direction: Direction,
) -> Fallible<S> {
    let lhs = S::exact(lhs)?;
    let rhs = S::exact(rhs)?;
    match operation {
        BinaryOperation::Add => lhs.add_round(rhs, direction),
        BinaryOperation::Sub => lhs.sub_round(rhs, direction),
        BinaryOperation::Mul => lhs.mul_round(rhs, direction),
        BinaryOperation::Div => lhs.div_round(rhs, direction),
    }
}

fn check_native_enclosures<R: NativeRegime>() -> Fallible<()> {
    let values = enclosure_values();
    let f64_max = rational(f64::MAX);
    for &lhs in &values {
        for &rhs in &values {
            for operation in [
                BinaryOperation::Add,
                BinaryOperation::Sub,
                BinaryOperation::Mul,
                BinaryOperation::Div,
            ] {
                if matches!(operation, BinaryOperation::Div) && rhs == 0.0 {
                    continue;
                }
                let exact = expected(lhs, rhs, operation);
                // A finite exact result outside the f64 range is covered by
                // the explicit range-error/overflow tests above. Here the
                // oracle checks the enclosure contract when both endpoints
                // are meaningful f64 bounds.
                if exact < -f64_max.clone() || exact > f64_max {
                    continue;
                }
                let down = native_result::<R>(lhs, rhs, operation, Direction::Down)?
                    .to_f64(Direction::Down)?;
                let up = native_result::<R>(lhs, rhs, operation, Direction::Up)?
                    .to_f64(Direction::Up)?;
                assert_encloses(down, up, exact);
            }
        }
    }
    Ok(())
}

fn check_software_enclosures<S: DirectedScalar>() -> Fallible<()> {
    let values = enclosure_values();
    let f64_max = rational(f64::MAX);
    for &lhs in &values {
        for &rhs in &values {
            for operation in [
                BinaryOperation::Add,
                BinaryOperation::Sub,
                BinaryOperation::Mul,
                BinaryOperation::Div,
            ] {
                if matches!(operation, BinaryOperation::Div) && rhs == 0.0 {
                    continue;
                }
                let exact = expected(lhs, rhs, operation);
                if exact < -f64_max.clone() || exact > f64_max {
                    continue;
                }
                let down = software_result::<S>(lhs, rhs, operation, Direction::Down)?
                    .to_f64(Direction::Down)?;
                let up = software_result::<S>(lhs, rhs, operation, Direction::Up)?
                    .to_f64(Direction::Up)?;
                assert_encloses(down, up, exact);
            }
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
enum ExtremeOutcome {
    Value(u64),
    Error(ErrorVariant),
}

fn extreme_outcome<S: DirectedTranscendental>(
    value: f64,
    direction: Direction,
    operation: impl FnOnce(S, Direction) -> Fallible<S>,
) -> ExtremeOutcome {
    match S::exact(value)
        .and_then(|value| operation(value, direction))
        .and_then(|value| value.to_f64(direction))
    {
        Ok(value) => ExtremeOutcome::Value(value.to_bits()),
        Err(error) => ExtremeOutcome::Error(error.variant),
    }
}

fn assert_extreme_unary_matches_rug(
    operation: &str,
    inputs: &[f64],
    evaluate: impl Fn(D, Direction) -> Fallible<D> + Copy,
    evaluate_rug: impl Fn(R, Direction) -> Fallible<R> + Copy,
) {
    for &input in inputs {
        for direction in [Direction::Down, Direction::Up] {
            assert_eq!(
                extreme_outcome(input, direction, evaluate),
                extreme_outcome(input, direction, evaluate_rug),
                "{operation}({input:?}) with {direction:?}"
            );
        }
    }
}

#[test]
fn test_extreme_transcendentals_match_rug() {
    assert_extreme_unary_matches_rug(
        "ln",
        &[
            -f64::MAX,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            1.0,
            f64::MAX,
        ],
        D::ln_round,
        R::ln_round,
    );
    assert_extreme_unary_matches_rug(
        "sqrt",
        &[
            -f64::MAX,
            -f64::from_bits(1),
            -0.0,
            0.0,
            f64::from_bits(1),
            f64::MIN_POSITIVE,
            f64::MAX,
        ],
        D::sqrt_round,
        R::sqrt_round,
    );
    assert_extreme_unary_matches_rug(
        "exp",
        &[-f64::MAX, -746.0, -745.0, -0.0, 0.0, 709.0, 710.0, f64::MAX],
        D::exp_round,
        R::exp_round,
    );
    assert_extreme_unary_matches_rug(
        "exp_m1",
        &[-f64::MAX, -38.0, -37.0, -0.0, 0.0, 709.0, 710.0, f64::MAX],
        D::exp_m1_round,
        R::exp_m1_round,
    );
}

#[test]
fn test_representative_arithmetic_enclosures() -> Fallible<()> {
    check_native_enclosures::<BestEffort>()?;
    check_native_enclosures::<Certified>()?;
    check_software_enclosures::<D>()?;
    check_software_enclosures::<R>()?;
    Ok(())
}
