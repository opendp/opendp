use super::*;
use crate::error::{ErrorVariant, Fallible};
use dashu::rational::RBig;

fn assert_error<T: std::fmt::Debug>(result: Fallible<T>, variant: ErrorVariant) {
    assert_eq!(result.unwrap_err().variant, variant);
}

fn check_ordering_and_unary<S: DirectedScalar + std::fmt::Debug>() -> Fallible<()> {
    let values = [
        f64::INFINITY,
        -0.0,
        -1.0,
        f64::from_bits(1),
        f64::NEG_INFINITY,
        0.0,
        1.0,
        -f64::from_bits(1),
        f64::MAX,
        -f64::MAX,
    ];
    let mut scalars = values
        .map(S::exact)
        .into_iter()
        .collect::<Fallible<Vec<_>>>()?;
    for (i, lhs) in scalars.iter().enumerate() {
        let negative: S = -lhs.clone();
        let absolute: S = lhs.clone().abs();
        for direction in [Direction::Down, Direction::Up] {
            assert_eq!(
                negative.to_f64(direction)?.to_bits(),
                (-values[i]).to_bits()
            );
            assert_eq!(
                absolute.to_f64(direction)?.to_bits(),
                values[i].abs().to_bits()
            );
        }
        assert_eq!(-negative, *lhs);
        for (j, rhs) in scalars.iter().enumerate() {
            let expected = values[i].partial_cmp(&values[j]).unwrap();
            assert_eq!(lhs.cmp(rhs), expected);
            assert_eq!(lhs.partial_cmp(rhs), Some(expected));
            assert_eq!(lhs == rhs, values[i] == values[j]);
            assert_eq!(lhs < rhs, values[i] < values[j]);
            assert_eq!(lhs <= rhs, values[i] <= values[j]);
        }
    }
    scalars.sort();
    let mut expected = values;
    expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for (actual, expected) in scalars.iter().zip(expected) {
        // Stable sorting must not normalize the stored sign of either zero.
        for direction in [Direction::Down, Direction::Up] {
            assert_eq!(actual.to_f64(direction)?.to_bits(), expected.to_bits());
        }
    }
    scalars.dedup();
    assert_eq!(scalars.len(), values.len() - 1);
    Ok(())
}

#[test]
fn test_native_ordering_and_unary() -> Fallible<()> {
    check_ordering_and_unary::<N64<Approximate>>()?;
    check_ordering_and_unary::<N64<Certified>>()
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
    Ok(())
}

#[derive(Clone, Copy, Debug)]
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
        2f64.powi(969), // just below the overflow midpoint when added to MAX
        2f64.powi(970), // overflow midpoint
        4.0,
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
    // Only the outward infinity is a valid bound for a finite rational.
    assert!(!down.is_nan() && down != f64::INFINITY);
    assert!(!up.is_nan() && up != f64::NEG_INFINITY);
    assert!(down <= up);
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
    if expected > rational(f64::MAX) {
        assert!(down.is_finite());
        assert_eq!(up, f64::INFINITY);
    } else if expected < rational(-f64::MAX) {
        assert_eq!(down, f64::NEG_INFINITY);
        assert!(up.is_finite());
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
    check_native_enclosures::<Certified>()?;
    Ok(())
}

#[test]
fn test_exact_native_identities() -> Fallible<()> {
    use BinaryOperation::*;
    for direction in [Direction::Down, Direction::Up] {
        for x in [
            0.0,
            -0.0,
            f64::from_bits(1),
            -f64::MIN_POSITIVE,
            3.5,
            -3.5,
            f64::MAX,
        ] {
            for (lhs, rhs, operation, expected) in [
                (x, 0.0, Add, x + 0.0),
                (x, -0.0, Add, x + -0.0),
                (0.0, x, Add, 0.0 + x),
                (-0.0, x, Add, -0.0 + x),
                (x, -x, Add, x + -x),
                (x, 0.0, Sub, x - 0.0),
                (x, -0.0, Sub, x - -0.0),
                (0.0, x, Sub, 0.0 - x),
                (x, x, Sub, x - x),
                (x, 0.0, Mul, x * 0.0),
                (x, -0.0, Mul, x * -0.0),
                (-0.0, x, Mul, -0.0 * x),
                (x, 1.0, Mul, x),
                (1.0, x, Mul, x),
                (-1.0, x, Mul, -x),
                (x, 1.0, Div, x),
                (x, -1.0, Div, -x),
            ] {
                let actual = native_result::<Certified>(lhs, rhs, operation, direction)?
                    .to_f64(direction)?;
                assert_eq!(
                    actual.to_bits(),
                    expected.to_bits(),
                    "{lhs:?} {operation:?} {rhs:?}"
                );
            }
            if x != 0.0 {
                for (lhs, rhs, expected) in [
                    (0.0, x, 0.0 / x),
                    (-0.0, x, -0.0 / x),
                    (x, x, 1.0),
                    (x, -x, -1.0),
                ] {
                    assert_eq!(
                        native_result::<Certified>(lhs, rhs, Div, direction)?
                            .to_f64(direction)?
                            .to_bits(),
                        expected.to_bits(),
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_inexact_zero_is_widened() -> Fallible<()> {
    // Preserve the known sign without collapsing a nonzero exact result to a
    // point interval. Exercise both halfway and smaller underflows.
    let minimum = f64::from_bits(1);
    for lhs_sign in [1.0, -1.0] {
        for rhs_sign in [1.0, -1.0] {
            for (rhs, operation) in [
                (0.5, BinaryOperation::Mul),
                (0.25, BinaryOperation::Mul),
                (2.0, BinaryOperation::Div),
                (4.0, BinaryOperation::Div),
            ] {
                let lhs = lhs_sign * minimum;
                let rhs = rhs_sign * rhs;
                let down = native_result::<Certified>(lhs, rhs, operation, Direction::Down)?
                    .to_f64(Direction::Down)?;
                let up = native_result::<Certified>(lhs, rhs, operation, Direction::Up)?
                    .to_f64(Direction::Up)?;
                let (expected_down, expected_up) = if lhs_sign == rhs_sign {
                    (0.0f64, minimum)
                } else {
                    (-minimum, -0.0f64)
                };
                assert_eq!(down.to_bits(), expected_down.to_bits());
                assert_eq!(up.to_bits(), expected_up.to_bits());
                assert_encloses(down, up, expected(lhs, rhs, operation));
            }
        }
    }
    Ok(())
}

#[test]
fn test_certified_sqrt() -> Fallible<()> {
    for x in enclosure_values().into_iter().filter(|x| *x >= 0.0) {
        let down = N64::<Certified>::exact(x)?
            .sqrt_round(Direction::Down)?
            .to_f64(Direction::Down)?;
        let up = N64::<Certified>::exact(x)?
            .sqrt_round(Direction::Up)?
            .to_f64(Direction::Up)?;
        assert!(down >= 0.0 && down <= up && up.is_finite());
        // Squaring exact rationals avoids an approximate square-root oracle.
        assert!(
            rational(down) * rational(down) <= rational(x),
            "sqrt lower bound at {x:?}"
        );
        assert!(
            rational(up) * rational(up) >= rational(x),
            "sqrt upper bound at {x:?}"
        );
    }
    for direction in [Direction::Down, Direction::Up] {
        for x in [0.0, -0.0, 1.0, f64::INFINITY] {
            assert_eq!(
                N64::<Certified>::exact(x)?
                    .sqrt_round(direction)?
                    .to_f64(direction)?
                    .to_bits(),
                x.to_bits(),
            );
        }
        for x in [-f64::from_bits(1), -1.0, f64::NEG_INFINITY] {
            assert_error(
                N64::<Certified>::exact(x)?.sqrt_round(direction),
                ErrorVariant::NumericDomain,
            );
        }
    }
    Ok(())
}

#[test]
fn test_native_extended_identities() -> Fallible<()> {
    for direction in [Direction::Down, Direction::Up] {
        for zero in [0.0, -0.0] {
            for numerator in [0.0, 1.0, f64::INFINITY] {
                assert_error(
                    N64::<Certified>::exact(numerator)?.div_round(N64::exact(zero)?, direction),
                    ErrorVariant::NumericIndeterminate,
                );
            }
        }
        for (lhs, rhs, operation) in [
            (f64::INFINITY, f64::INFINITY, BinaryOperation::Sub),
            (0.0, f64::NEG_INFINITY, BinaryOperation::Mul),
            (f64::NEG_INFINITY, f64::INFINITY, BinaryOperation::Div),
        ] {
            assert_error(
                native_result::<Certified>(lhs, rhs, operation, direction),
                ErrorVariant::NumericIndeterminate,
            );
        }
        for (lhs, rhs, operation, expected) in [
            (f64::INFINITY, 0.0, BinaryOperation::Add, f64::INFINITY),
            (
                f64::NEG_INFINITY,
                1.0,
                BinaryOperation::Mul,
                f64::NEG_INFINITY,
            ),
            (1.0, f64::NEG_INFINITY, BinaryOperation::Div, -0.0),
        ] {
            assert_eq!(
                native_result::<Certified>(lhs, rhs, operation, direction)?
                    .to_f64(direction)?
                    .to_bits(),
                expected.to_bits()
            );
        }
    }
    Ok(())
}
