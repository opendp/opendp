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
        for zero in [0.0, -0.0] {
            for value in [
                S::exact(zero)?,
                S::exact(zero)?.sqrt_round(direction)?,
                S::exact(zero)?.exp_m1_round(direction)?,
            ] {
                assert_eq!(value.to_f64(direction)?.to_bits(), zero.to_bits());
            }
            assert_error(
                S::exact(1.0)?.div_round(S::exact(zero)?, direction),
                ErrorVariant::NumericIndeterminate,
            );
        }
        assert_eq!(
            S::exact(1.0)?
                .div_round(S::exact(f64::NEG_INFINITY)?, direction)?
                .to_f64(direction)?
                .to_bits(),
            (-0.0f64).to_bits(),
        );
        // Exact intermediate results can carry precision zero in Dashu.
        assert_eq!(
            S::exact(0.0)?
                .exp_round(direction)?
                .exp_round(direction)?
                .to_f64(direction)?,
            S::exact(1.0)?.exp_round(direction)?.to_f64(direction)?,
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
    let negative = -positive;
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

    let negative = -positive;
    assert_eq!(negative.to_f64(Direction::Down)?, -f64::from_bits(1));
    assert_eq!(negative.to_f64(Direction::Up)?, -0.0);

    // Halfway cases at several subnormal magnitudes, including the normal
    // boundary, exercise Dashu's native directed conversion without corrections.
    let unit = f64::from_bits(1);
    for magnitude in [0.5, 1.5, 2.5, (1u64 << 52) as f64 - 0.5] {
        for sign in [-1.0, 1.0] {
            let factor = sign * magnitude;
            let value = S::exact(unit)?.mul_round(S::exact(factor)?, Direction::Down)?;
            let down = value.to_f64(Direction::Down)?;
            let up = value.to_f64(Direction::Up)?;
            assert_encloses(down, up, rational(unit) * rational(factor));
            assert_eq!(down.next_up(), up);
        }
    }
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
    assert!(sum == S::exact(5.0)?);
    Ok(())
}

#[test]
fn test_consuming_arithmetic_and_comparison() -> Fallible<()> {
    check_consuming_arithmetic_and_comparison::<D>()?;
    check_consuming_arithmetic_and_comparison::<R>()
}

fn check_software_ordering<S: DirectedTranscendental + std::fmt::Debug>() -> Fallible<()> {
    check_ordering_and_unary::<S>()?;
    let one = S::exact(1.0)?;
    let tiny = S::exact(2.0f64.powi(-255))?;
    let below = one.clone().sub_round(tiny.clone(), Direction::Down)?;
    let above = one.clone().add_round(tiny, Direction::Up)?;
    assert!(below < one && one < above);
    assert_eq!(below.to_f64(Direction::Up)?, 1.0);
    assert_eq!(above.to_f64(Direction::Down)?, 1.0);
    assert!(-above < -one.clone());
    assert_eq!(
        S::exact(2.0)?.mul_round(S::exact(0.5)?, Direction::Up)?,
        one
    );

    // Ordering must use internal values even when f64 narrowing underflows
    // or overflows, rather than reporting conversion range errors.
    let small = S::exact(-10_000.0)?.exp_round(Direction::Up)?;
    assert!(small > S::exact(0.0)?);
    assert!(-small.clone() < S::exact(-0.0)?);
    assert_eq!(small.to_f64(Direction::Down)?, 0.0);
    let large = S::exact(10_000.0)?.exp_round(Direction::Up)?;
    assert!(S::exact(f64::MAX)? < large && large < S::exact(f64::INFINITY)?);
    assert!(-large > S::exact(f64::NEG_INFINITY)?);
    Ok(())
}

#[test]
fn test_software_ordering_and_unary() -> Fallible<()> {
    check_software_ordering::<D>()?;
    check_software_ordering::<R>()
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

fn software_bound(result: Fallible<f64>, direction: Direction, exact: &RBig) -> f64 {
    match result {
        Ok(value) => value,
        Err(error) => match (&error.variant, direction) {
            (ErrorVariant::NumericRangeAbove, Direction::Up) if *exact > rational(f64::MAX) => {
                f64::INFINITY
            }
            (ErrorVariant::NumericRangeBelow, Direction::Down) if *exact < rational(-f64::MAX) => {
                f64::NEG_INFINITY
            }
            _ => panic!("unexpected narrowing error: {error:?}"),
        },
    }
}

fn check_software_enclosures<S: DirectedScalar>() -> Fallible<()> {
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
                let down = software_bound(
                    software_result::<S>(lhs, rhs, operation, Direction::Down)?
                        .to_f64(Direction::Down),
                    Direction::Down,
                    &exact,
                );
                let up = software_bound(
                    software_result::<S>(lhs, rhs, operation, Direction::Up)?.to_f64(Direction::Up),
                    Direction::Up,
                    &exact,
                );
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
        &[-746.0, -745.0, -0.0, 0.0, 709.0, 710.0],
        D::exp_round,
        R::exp_round,
    );
    assert_extreme_unary_matches_rug(
        "exp_m1",
        &[-f64::MAX, -38.0, -37.0, -0.0, 0.0, 709.0, 710.0],
        D::exp_m1_round,
        R::exp_m1_round,
    );
}

// Transfer the exact binary significand and exponent, never a narrowed f64.
// 1024 bits represent every tested 256-bit Dashu endpoint without rounding.
fn dashu_as_high_precision_rug(value: &D) -> R {
    let repr = value.repr.repr();
    assert!(repr.digits() <= 256, "expected at most 256 binary digits");
    let significand = rug::Float::parse(repr.significand().to_string()).unwrap();
    let mut repr_rug = rug::Float::with_val(1024, significand);
    repr_rug <<= i32::try_from(repr.exponent()).unwrap();
    R { repr: repr_rug }
}

fn assert_internal_transcendental_enclosure(
    operation: &str,
    input: D,
    evaluate: impl Fn(D, Direction) -> Fallible<D>,
    evaluate_rug: impl Fn(R, Direction) -> Fallible<R>,
) -> Fallible<()> {
    for direction in [Direction::Down, Direction::Up] {
        let actual = evaluate(input.clone(), direction)?;
        let reference = evaluate_rug(dashu_as_high_precision_rug(&input), direction)?;
        let endpoint = dashu_as_high_precision_rug(&actual);
        assert!(endpoint.repr.is_finite() && reference.repr.is_finite());
        assert!(
            match direction {
                Direction::Down => endpoint.repr <= reference.repr,
                Direction::Up => endpoint.repr >= reference.repr,
            },
            "{operation}({input:?}) with {direction:?}: internal Dashu bound excludes MPFR bound",
        );
    }
    Ok(())
}

#[test]
fn test_internal_transcendental_enclosures() -> Fallible<()> {
    let tiny = 2.0f64.powi(-255);
    for x in [
        -10_000.0,
        -1.0,
        -tiny,
        -f64::from_bits(1),
        0.0,
        f64::from_bits(1),
        tiny,
        std::f64::consts::LN_2.next_down(),
        std::f64::consts::LN_2,
        std::f64::consts::LN_2.next_up(),
        1.0,
        710.0,
        10_000.0,
    ] {
        assert_internal_transcendental_enclosure("exp", D::exact(x)?, D::exp_round, R::exp_round)?;
        assert_internal_transcendental_enclosure(
            "expm1",
            D::exact(x)?,
            D::exp_m1_round,
            R::exp_m1_round,
        )?;
    }
    for x in [
        f64::from_bits(1),
        f64::MIN_POSITIVE,
        0.5,
        1.0f64.next_down(),
        1.0,
        1.0f64.next_up(),
        2.0,
        f64::MAX,
    ] {
        assert_internal_transcendental_enclosure("ln", D::exact(x)?, D::ln_round, R::ln_round)?;
    }
    // These operands differ from one internally but collapse to one in f64.
    // ln must retain their cancellation-sensitive, nonzero answers.
    for offset in [-tiny, tiny] {
        let input = D::exact(1.0)?.add_round(D::exact(offset)?, Direction::Down)?;
        assert_internal_transcendental_enclosure("ln", input, D::ln_round, R::ln_round)?;
    }
    Ok(())
}

#[test]
fn test_software_sqrt_exponent_boundary() -> Fallible<()> {
    for direction in [Direction::Down, Direction::Up] {
        let mut tiny = D::exact(0.5)?;
        for _ in 0..isize::BITS - 2 {
            tiny = tiny.clone().mul_round(tiny, direction)?;
        }
        tiny = tiny.mul_round(D::exact(2.0)?, direction)?;
        tiny = tiny.clone().mul_round(tiny, direction)?;
        assert_error(tiny.sqrt_round(direction), ErrorVariant::NumericBackend);
    }
    Ok(())
}

#[test]
fn test_software_provider_exhaustion() -> Fallible<()> {
    // A finite result outside Dashu's exponent range is a backend failure, not
    // a genuine infinity or an f64 narrowing error. Do not saturate it silently.
    for direction in [Direction::Down, Direction::Up] {
        for x in [-f64::MAX, f64::MAX] {
            assert_error(
                D::exact(x)?.exp_round(direction),
                ErrorVariant::NumericBackend,
            );
        }
        assert_error(
            D::exact(f64::MAX)?.exp_m1_round(direction),
            ErrorVariant::NumericBackend,
        );
    }
    Ok(())
}

#[test]
fn test_representative_arithmetic_enclosures() -> Fallible<()> {
    check_native_enclosures::<Certified>()?;
    check_software_enclosures::<D>()?;
    check_software_enclosures::<R>()?;
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

fn check_sqrt_enclosures<S: DirectedScalar>() -> Fallible<()> {
    for x in enclosure_values().into_iter().filter(|x| *x >= 0.0) {
        let down = S::exact(x)?
            .sqrt_round(Direction::Down)?
            .to_f64(Direction::Down)?;
        let up = S::exact(x)?
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
    Ok(())
}

#[test]
fn test_software_sqrt_enclosures() -> Fallible<()> {
    check_sqrt_enclosures::<D>()?;
    check_sqrt_enclosures::<R>()
}

#[test]
fn test_certified_sqrt() -> Fallible<()> {
    check_sqrt_enclosures::<N64<Certified>>()?;
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
