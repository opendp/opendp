use super::*;
use crate::error::{ErrorVariant, Fallible};

/// Pins `(lower, upper)` bounds, and checks they are no looser than `old`,
/// the bounds from the workarounds used before dashu 0.6.
fn check(actual: (Fallible<f64>, Fallible<f64>), pinned: (f64, f64), old: (f64, f64)) {
    assert_eq!((actual.0.unwrap(), actual.1.unwrap()), pinned);
    assert!(old.0 <= pinned.0 && pinned.1 <= old.1);
}

#[test]
fn test_inf_sqrt() {
    let sqrt = |x: f64| (x.neg_inf_sqrt(), x.inf_sqrt());
    let b = (2.449489742783178, 2.4494897427831783);
    check(sqrt(6.0), b, b);
    let b = (0.31622776601683794, 0.316227766016838);
    check(sqrt(0.1), b, b);
    let b = (4.898979485566356, 4.898979485566357);
    check(sqrt(24.0), b, b);
}

#[test]
fn test_inf_log2() {
    let log2 = |x: f64| (x.neg_inf_log2(), x.inf_log2());
    check(
        log2(0.1),
        (-3.3219280948873626, -3.321928094887362),
        (-3.3219339847564697, -3.3219258785247803),
    );
    check(
        log2(3.0),
        (1.584962500721156, 1.5849625007211563),
        (1.584962248802185, 1.5849627256393433),
    );
}

#[test]
fn test_inf_exp() {
    let exp = |x: f64| (x.neg_inf_exp(), x.inf_exp());
    let exp_m1 = |x: f64| (x.neg_inf_exp_m1(), x.inf_exp_m1());
    for x in [-746.0, -1e4] {
        check(exp(x), (0.0, 5e-324), (-5e-324, 5e-324));
    }
    for x in [-40.0, -1e4] {
        let b = (-1.0, -0.9999999999999999);
        check(exp_m1(x), b, b);
    }
}

#[test]
fn test_inf_powi() {
    let powi = |x: f64, p: IBig| (x.neg_inf_powi(p.clone()), x.inf_powi(p));
    let huge = IBig::from(10u8).pow(16);
    check(powi(0.5, huge.clone()), (0.0, 5e-324), (-5e-324, 5e-324));
    check(powi(-0.5, huge + 1), (-5e-324, -0.0), (-5e-324, 5e-324));
}

fn assert_error<T: std::fmt::Debug>(result: Fallible<T>, variant: ErrorVariant) {
    assert_eq!(result.unwrap_err().variant, variant);
}

#[test]
fn test_integer_error_taxonomy_and_rounding() {
    macro_rules! check_integer {
        ($($ty:ty),+) => {$(
            assert_error((1 as $ty).alerting_div(&0), ErrorVariant::NumericError);
            assert_error((1 as $ty).inf_div(&0), ErrorVariant::NumericError);
            assert_error((0 as $ty).neg_inf_div(&0), ErrorVariant::NumericError);
            assert_error(<$ty>::MAX.alerting_add(&1), ErrorVariant::Overflow);
            assert_eq!((7 as $ty).inf_div(&3).unwrap(), 3);
            assert_eq!((7 as $ty).neg_inf_div(&3).unwrap(), 2);
        )+};
    }
    check_integer!(
        u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
    );
    macro_rules! check_signed {
        ($($ty:ty),+) => {$(
            assert_error(<$ty>::MIN.alerting_div(&-1), ErrorVariant::Overflow);
            assert_error(<$ty>::MIN.inf_div(&-1), ErrorVariant::Overflow);
            assert_error(<$ty>::MIN.neg_inf_div(&-1), ErrorVariant::Overflow);
            // All sign combinations, exact and inexact quotients.
            for (lhs, rhs, down, up) in [
                (7, 3, 2, 3), (-7, 3, -3, -2),
                (7, -3, -3, -2), (-7, -3, 2, 3),
                (6, -3, -2, -2), (-6, 3, -2, -2),
            ] {
                assert_eq!((lhs as $ty).neg_inf_div(&(rhs as $ty)).unwrap(), down);
                assert_eq!((lhs as $ty).inf_div(&(rhs as $ty)).unwrap(), up);
            }
        )+};
    }
    check_signed!(i8, i16, i32, i64, i128, isize);
}

#[test]
fn test_legacy_float_error_taxonomy() {
    macro_rules! check_float {
        ($ty:ty) => {{
            let zero = 0.0 as $ty;
            let one = 1.0 as $ty;
            let negative = -1.0 as $ty;
            assert_error(<$ty>::NAN.alerting_abs(), ErrorVariant::NumericError);
            assert_eq!(<$ty>::NEG_INFINITY.alerting_abs().unwrap(), <$ty>::INFINITY);
            for result in [
                one.alerting_div(&zero),
                zero.alerting_div(&zero),
                one.inf_div(&zero),
                zero.neg_inf_div(&zero),
                <$ty>::INFINITY.alerting_sub(&<$ty>::INFINITY),
                <$ty>::INFINITY.inf_sub(&<$ty>::INFINITY),
                <$ty>::INFINITY.neg_inf_add(&<$ty>::NEG_INFINITY),
                zero.alerting_mul(&<$ty>::INFINITY),
                zero.inf_mul(&<$ty>::INFINITY),
                <$ty>::INFINITY.neg_inf_div(&<$ty>::INFINITY),
                zero.inf_ln(),
                zero.neg_inf_ln(),
                zero.inf_log2(),
                zero.neg_inf_log2(),
                negative.inf_ln_1p(),
                negative.neg_inf_ln_1p(),
                zero.alerting_pow(&negative),
                zero.inf_powi(IBig::from(-1)),
                zero.neg_inf_powi(IBig::from(-1)),
            ] {
                assert_error(result, ErrorVariant::NumericError);
            }
            for result in [
                negative.inf_sqrt(),
                negative.neg_inf_sqrt(),
                negative.inf_ln(),
                negative.neg_inf_ln(),
                negative.inf_log2(),
                negative.neg_inf_log2(),
                (-2.0 as $ty).inf_ln_1p(),
                (-2.0 as $ty).neg_inf_ln_1p(),
                negative.alerting_pow(&(0.5 as $ty)),
            ] {
                assert_error(result, ErrorVariant::NumericError);
            }
            for result in [
                <$ty>::MAX.alerting_add(&<$ty>::MAX),
                (-<$ty>::MAX).alerting_sub(&<$ty>::MAX),
                <$ty>::MAX.alerting_mul(&(2.0 as $ty)),
                <$ty>::MAX.alerting_div(&(0.5 as $ty)),
                <$ty>::MAX.alerting_pow(&(2.0 as $ty)),
                <$ty>::MAX.inf_add(&<$ty>::MAX),
                (-<$ty>::MAX).neg_inf_add(&(-<$ty>::MAX)),
                (-<$ty>::MAX).neg_inf_sub(&<$ty>::MAX),
                <$ty>::MAX.inf_mul(&(2.0 as $ty)),
                (-<$ty>::MAX).neg_inf_mul(&(2.0 as $ty)),
                <$ty>::MAX.inf_div(&(0.5 as $ty)),
                (-<$ty>::MAX).neg_inf_div(&(0.5 as $ty)),
                (1000.0 as $ty).inf_exp(),
                (1000.0 as $ty).inf_exp_m1(),
                (2.0 as $ty).inf_powi(IBig::from(2048)),
                (-2.0 as $ty).neg_inf_powi(IBig::from(2049)),
                (0.5 as $ty).inf_powi(IBig::from(-2048)),
                (-0.5 as $ty).neg_inf_powi(IBig::from(-2049)),
            ] {
                assert_error(result, ErrorVariant::Overflow);
            }
            // A finite mathematical result outside the float range still has an inward bound.
            for result in [
                <$ty>::MAX.neg_inf_add(&<$ty>::MAX),
                <$ty>::MAX.neg_inf_sub(&(-<$ty>::MAX)),
                <$ty>::MAX.neg_inf_mul(&(2.0 as $ty)),
                (-<$ty>::MAX).neg_inf_mul(&(-2.0 as $ty)),
                <$ty>::MAX.neg_inf_div(&(0.5 as $ty)),
                (-<$ty>::MAX).neg_inf_div(&(-0.5 as $ty)),
                (1000.0 as $ty).neg_inf_exp(),
                (1000.0 as $ty).neg_inf_exp_m1(),
                (2.0 as $ty).neg_inf_powi(IBig::from(2048)),
                (0.5 as $ty).neg_inf_powi(IBig::from(-2048)),
            ] {
                assert_eq!(result.unwrap(), <$ty>::MAX);
            }
            for result in [
                (-<$ty>::MAX).inf_add(&(-<$ty>::MAX)),
                (-<$ty>::MAX).inf_sub(&<$ty>::MAX),
                (-<$ty>::MAX).inf_mul(&(2.0 as $ty)),
                <$ty>::MAX.inf_mul(&(-2.0 as $ty)),
                (-<$ty>::MAX).inf_div(&(0.5 as $ty)),
                <$ty>::MAX.inf_div(&(-0.5 as $ty)),
                (-2.0 as $ty).inf_powi(IBig::from(2049)),
                (-0.5 as $ty).inf_powi(IBig::from(-2049)),
            ] {
                assert_eq!(result.unwrap(), <$ty>::MIN);
            }
            for divisor in [zero, -zero] {
                for result in [
                    zero.alerting_div(&divisor),
                    zero.inf_div(&divisor),
                    zero.neg_inf_div(&divisor),
                ] {
                    assert_eq!(
                        result.unwrap_err().message.as_deref(),
                        Some("division by zero")
                    );
                }
            }
            for invalid in [<$ty>::NAN, <$ty>::INFINITY, <$ty>::NEG_INFINITY] {
                for result in [
                    invalid.alerting_add(&one),
                    one.alerting_add(&invalid),
                    invalid.alerting_sub(&one),
                    one.alerting_sub(&invalid),
                    invalid.alerting_mul(&one),
                    one.alerting_mul(&invalid),
                    invalid.alerting_div(&one),
                    one.alerting_div(&invalid),
                    invalid.alerting_pow(&one),
                    one.alerting_pow(&invalid),
                    invalid.inf_add(&one),
                    one.inf_add(&invalid),
                    invalid.neg_inf_add(&one),
                    one.neg_inf_add(&invalid),
                    invalid.inf_sub(&one),
                    one.inf_sub(&invalid),
                    invalid.neg_inf_sub(&one),
                    one.neg_inf_sub(&invalid),
                    invalid.inf_mul(&one),
                    one.inf_mul(&invalid),
                    invalid.neg_inf_mul(&one),
                    one.neg_inf_mul(&invalid),
                    invalid.inf_div(&one),
                    one.inf_div(&invalid),
                    invalid.neg_inf_div(&one),
                    one.neg_inf_div(&invalid),
                    invalid.inf_sqrt(),
                    invalid.neg_inf_sqrt(),
                    invalid.inf_ln(),
                    invalid.neg_inf_ln(),
                    invalid.inf_log2(),
                    invalid.neg_inf_log2(),
                    invalid.inf_ln_1p(),
                    invalid.neg_inf_ln_1p(),
                    invalid.inf_exp(),
                    invalid.neg_inf_exp(),
                    invalid.inf_exp_m1(),
                    invalid.neg_inf_exp_m1(),
                    invalid.inf_powi(IBig::ONE),
                    invalid.neg_inf_powi(IBig::ONE),
                ] {
                    assert_error(result, ErrorVariant::NumericError);
                }
            }
        }};
    }
    check_float!(f32);
    check_float!(f64);
}
