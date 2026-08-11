use super::*;
use crate::error::{ErrorVariant, Fallible};
use dashu::rational::RBig;

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
    // Division by zero is intentionally classified as indeterminate: the
    // directed scalar API does not assign an extended-real result to it.
    assert_error(
        N64::<Certified>::exact(1.0)?.div_round(N64::exact(0.0)?, Direction::Down),
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

#[test]
fn test_representative_arithmetic_enclosures() -> Fallible<()> {
    check_native_enclosures::<BestEffort>()?;
    check_native_enclosures::<Certified>()?;
    Ok(())
}
