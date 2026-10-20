use super::*;
use crate::error::Fallible;

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
