use super::*;
use crate::error::Fallible;
use dashu::rational::RBig;

/// f64 values spanning signs, magnitudes, subnormals, and exact zero
/// (exp of exact zero regressed to a panic under dashu 0.5 before the
///  precision fix in this module).
const GRID: [f64; 15] = [
    -700.0, -2.5, -1.5, -0.5, -2.2e-16, -5e-324, 0.0, 5e-324, 2.2e-16, 0.1, 0.5, 1.0, 1.5, 2.5,
    700.0,
];

fn rat(v: f64) -> RBig {
    RBig::try_from(v).unwrap()
}

/// Assert `lo_result <= reference <= hi_result` in exact rational arithmetic.
fn assert_brackets(lo_result: f64, hi_result: f64, reference: &RBig, context: &str) {
    assert!(
        &rat(lo_result) <= reference,
        "{context}: neg_inf result {lo_result:e} exceeds the true value {:e}",
        reference.to_f64().value()
    );
    assert!(
        reference <= &rat(hi_result),
        "{context}: inf result {hi_result:e} is below the true value {:e}",
        reference.to_f64().value()
    );
}

/// The directed arithmetic must bracket the exact rational result.
#[test]
fn test_inf_binary_containment() -> Fallible<()> {
    for x in GRID {
        for y in GRID {
            let (rx, ry) = (rat(x), rat(y));
            assert_brackets(x.neg_inf_add(&y)?, x.inf_add(&y)?, &(&rx + &ry), "add");
            assert_brackets(x.neg_inf_sub(&y)?, x.inf_sub(&y)?, &(&rx - &ry), "sub");
            assert_brackets(x.neg_inf_mul(&y)?, x.inf_mul(&y)?, &(&rx * &ry), "mul");
            // division by zero and f64-overflowing quotients (e.g. -700 / -5e-324)
            // error by contract; containment applies to representable results
            if y != 0.0 && (x / y).is_finite() {
                assert_brackets(x.neg_inf_div(&y)?, x.inf_div(&y)?, &(&rx / &ry), "div");
            }
        }
    }
    Ok(())
}

/// Integer powers have exact rational references.
#[test]
fn test_inf_powi_containment() -> Fallible<()> {
    for x in GRID {
        for p in [0u32, 1, 2, 3, 7] {
            let reference = rat(x).pow(p as usize);
            assert_brackets(
                x.neg_inf_powi(IBig::from(p))?,
                x.inf_powi(IBig::from(p))?,
                &reference,
                "powi",
            );
        }
    }
    Ok(())
}

/// Certified two-sided reference bracket for a transcendental function,
/// computed at 150-bit precision with directed rounding carried through.
fn ref_bracket(
    x: f64,
    f_down: impl Fn(FBig<Down>) -> FBig<Down>,
    f_up: impl Fn(FBig<Up>) -> FBig<Up>,
) -> (RBig, RBig) {
    let lo = f_down(
        FBig::<Down>::try_from(x)
            .unwrap()
            .with_precision(150)
            .value(),
    );
    let hi = f_up(FBig::<Up>::try_from(x).unwrap().with_precision(150).value());
    (RBig::try_from(lo).unwrap(), RBig::try_from(hi).unwrap())
}

/// Assert the f64 directed results bracket the 150-bit reference bracket:
/// `lo_result <= ref_lo <= ref_hi <= hi_result` (up to representability at
/// the reference precision, far finer than any f64 gap).
fn assert_brackets_ref(lo_result: f64, hi_result: f64, ref_lo: &RBig, ref_hi: &RBig, ctx: &str) {
    assert!(
        &rat(lo_result) <= ref_lo,
        "{ctx}: neg_inf result {lo_result:e} exceeds the reference lower bound"
    );
    assert!(
        ref_hi <= &rat(hi_result),
        "{ctx}: inf result {hi_result:e} is below the reference upper bound"
    );
}

#[test]
fn test_inf_exp_containment() -> Fallible<()> {
    for x in GRID {
        let (ref_lo, ref_hi) = ref_bracket(x, |v| v.exp(), |v| v.exp());
        assert_brackets_ref(x.neg_inf_exp()?, x.inf_exp()?, &ref_lo, &ref_hi, "exp");

        let (ref_lo, ref_hi) = ref_bracket(x, |v| v.exp_m1(), |v| v.exp_m1());
        assert_brackets_ref(
            x.neg_inf_exp_m1()?,
            x.inf_exp_m1()?,
            &ref_lo,
            &ref_hi,
            "exp_m1",
        );
    }
    Ok(())
}

/// Below the guard thresholds the true value cannot be computed at the
/// reference precision, but the returned constants bound it by construction.
/// Generic over the float type; each test below supplies its type's cases.
fn check_exp_guard_containment<T: crate::traits::Float>(
    below_guard: &[T],
    min_subnormal: T,
    neg_one_next_up: T,
) -> Fallible<()> {
    for &x in below_guard {
        // 0 < exp(x) < min subnormal
        assert_eq!(x.neg_inf_exp()?, T::zero());
        assert_eq!(x.inf_exp()?, min_subnormal);

        // -1 < exp_m1(x) < -1 + one ulp of 1
        assert_eq!(x.neg_inf_exp_m1()?, -T::one());
        assert_eq!(x.inf_exp_m1()?, neg_one_next_up);
    }
    Ok(())
}

#[test]
fn test_inf_exp_guard_containment_f64() -> Fallible<()> {
    check_exp_guard_containment(
        &[-746.0f64, -1e4, -1e19, -1e30, f64::MIN],
        f64::from_bits(1),
        (-1.0f64).next_up(),
    )
}

#[test]
fn test_inf_exp_guard_containment_f32() -> Fallible<()> {
    check_exp_guard_containment(
        &[-105.0f32, -1e4, -1e19, -1e30, f32::MIN],
        f32::from_bits(1),
        (-1.0f32).next_up(),
    )
}

#[test]
fn test_inf_ln_sqrt_containment() -> Fallible<()> {
    for x in GRID {
        if x > 0.0 {
            let (ref_lo, ref_hi) = ref_bracket(x, |v| v.ln(), |v| v.ln());
            assert_brackets_ref(x.neg_inf_ln()?, x.inf_ln()?, &ref_lo, &ref_hi, "ln");
        }
        if x >= 0.0 {
            let (ref_lo, ref_hi) = ref_bracket(x, |v| v.sqrt(), |v| v.sqrt());
            assert_brackets_ref(x.neg_inf_sqrt()?, x.inf_sqrt()?, &ref_lo, &ref_hi, "sqrt");
        }
        if x > -1.0 {
            let (ref_lo, ref_hi) = ref_bracket(x, |v| v.ln_1p(), |v| v.ln_1p());
            assert_brackets_ref(
                x.neg_inf_ln_1p()?,
                x.inf_ln_1p()?,
                &ref_lo,
                &ref_hi,
                "ln_1p",
            );
        }
    }
    Ok(())
}

/// The paths added for dashu 0.5 compatibility, exercised directly.
#[test]
fn test_inf_powi_saturation_paths() -> Fallible<()> {
    // negative IBig exponent: exact rational reference (also exercises dashu's
    // limited-precision assertion path, which precision-0 zeros used to trip)
    for x in [0.5f64, 1.5, 2.5, -1.5] {
        for p in [1i32, 2, 3] {
            let reference = RBig::ONE / rat(x).pow(p as usize);
            assert_brackets(
                x.neg_inf_powi(IBig::from(-p))?,
                x.inf_powi(IBig::from(-p))?,
                &reference,
                "powi negative exponent",
            );
        }
    }
    // zero base with negative exponent fails closed on both sides
    assert!(0.0f64.inf_powi(IBig::from(-2)).is_err());
    assert!(0.0f64.neg_inf_powi(IBig::from(-2)).is_err());

    // astronomically large exponents saturate in dashu; the corrections must
    // keep the returned bounds on the conservative side of the true value
    let huge = IBig::from(10u8).pow(16);
    // |base| < 1: true result in (0, min subnormal); the saturated zero may be
    // nudged one further step down by the directed cast
    assert_eq!(0.5f64.inf_powi(huge.clone())?, f64::from_bits(1));
    let down = 0.5f64.neg_inf_powi(huge.clone())?;
    assert!((-f64::from_bits(1)..=0.0).contains(&down), "got {down:e}");
    // negative |base| < 1 with odd exponent: true result in (-min subnormal, 0);
    // the directed casts may nudge the saturated signed zero one step outward
    let huge_odd = IBig::from(10u8).pow(16) + IBig::ONE;
    let up = (-0.5f64).inf_powi(huge_odd.clone())?;
    assert!(
        (0.0..=f64::from_bits(1)).contains(&up),
        "upper bound must be >= the (negative) true value and tight, got {up:e}"
    );
    // no f64 exists strictly between -min subnormal and 0, so the only sound
    // (and tight) lower bound is -min subnormal itself
    assert_eq!((-0.5f64).neg_inf_powi(huge_odd)?, -f64::from_bits(1));
    // |base| > 1: pre-0.5 this panicked into Err; dashu now saturates to
    // infinity, which the casts turn into the widest sound finite bounds
    assert_eq!(2.0f64.neg_inf_powi(IBig::from(10u8).pow(16))?, f64::MAX);
    assert!(2.0f64.inf_powi(IBig::from(10u8).pow(16)).is_err());
    Ok(())
}

/// exp_m1 in the cancellation band just above the guard threshold, where the
/// true value is within a few ulps of -1.
#[test]
fn test_inf_exp_m1_near_threshold() -> Fallible<()> {
    for x in [-36.9f64, -36.5, -36.0, -30.0] {
        let (ref_lo, ref_hi) = ref_bracket(x, |v| v.exp_m1(), |v| v.exp_m1());
        assert_brackets_ref(
            x.neg_inf_exp_m1()?,
            x.inf_exp_m1()?,
            &ref_lo,
            &ref_hi,
            "exp_m1 near threshold",
        );
    }
    Ok(())
}
