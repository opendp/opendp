use std::thread::{self, JoinHandle};

use dashu::{integer::IBig, rational::RBig, rbig, ubig};

use crate::{
    domains::AtomDomain,
    measurements::make_laplace,
    measures::MaxDivergence,
    metrics::AbsoluteDistance,
    traits::{
        ExactIntCast,
        samplers::{sample_discrete_gaussian, sample_uniform_uint_below},
    },
};

use super::*;
use crate::test_rounding::{Interval, assert_rounds_up};

fn test_laplace_tail(tail: UBig, theoretical_alpha: f64, label: &str) -> Fallible<()> {
    let tail = i8::try_from(tail)?;
    let scale = 1.;

    println!("alpha: {}", theoretical_alpha);
    let m_dlap = make_laplace::<AtomDomain<i8>, AbsoluteDistance<i8>, MaxDivergence>(
        Default::default(),
        Default::default(),
        scale,
        None,
    )?;
    let n = 50_000;
    let empirical_alpha = (0..n)
        .filter(|_| m_dlap.invoke(&0).unwrap().clamp(-127, 127) > tail as i8)
        .count() as f64
        / n as f64;

    println!("{} significance levels/alpha", label);
    println!("Theoretical: {:?}", theoretical_alpha);
    println!("Empirical:   {:?}f", empirical_alpha);
    // this test has a small likelihood of failing
    assert!((empirical_alpha - theoretical_alpha).abs() < 1e-2);
    Ok(())
}

#[test]
pub fn test_empirical_integrate_discrete_laplace_tail_fixed() -> Fallible<()> {
    let scale = rbig!(1);
    let tail = ubig!(2);
    let alpha = conservative_discrete_laplacian_tail_to_alpha(scale, tail.clone())?;
    test_laplace_tail(tail, alpha, "Discrete Laplace")
}

#[test]
pub fn test_empirical_integrate_discrete_laplace_tail_random() -> Fallible<()> {
    let scale = rbig!(1);
    let tail = UBig::from(1u32 + sample_uniform_uint_below(10)?);
    let alpha = conservative_discrete_laplacian_tail_to_alpha(scale, tail.clone())?;
    test_laplace_tail(tail, alpha, "Discrete Laplace")
}

/// Computes the probability of sampling a value greater than `tail` from the discrete gaussian distribution.
pub(super) fn discrete_gaussian_tail_to_alpha(scale: f64, tail: u32) -> Fallible<f64> {
    let mut total = 0.;
    let mut x = i32::exact_int_cast(tail)?;
    loop {
        x += 1;
        let dens = dg_pdf(x, scale);
        if dens.is_zero() {
            return Ok(total / dg_normalization_term(scale));
        }
        total += dens;
    }
}

fn test_gaussian_tail(scale: f64, tail: u32) -> Fallible<()> {
    let alpha_upper =
        conservative_discrete_gaussian_tail_to_alpha(RBig::try_from(scale)?, UBig::from(tail))?;
    let alpha_avg = discrete_gaussian_tail_to_alpha(scale, tail)?;

    let r_scale = RBig::try_from(scale)?;
    let i_tail = IBig::from(tail);
    let n = 50_000;
    let alpha_emp = (0..n)
        .filter(|_| sample_discrete_gaussian(r_scale.clone()).unwrap() > i_tail)
        .count() as f64
        / n as f64;

    println!("Discrete Gaussian significance levels/alpha");
    println!("Theoretical (upper): {:?}", alpha_upper);
    println!("Theoretical (avg):   {:?}", alpha_avg);
    println!("Empirical:           {:?}", alpha_emp);

    // This test has a small likelihood of failing.
    // alpha_upper can be loose by a large factor when scale is small,
    // as it uses the tail bound of the continuous gaussian.
    assert!(alpha_emp < alpha_upper);
    Ok(())
}

#[test]
pub fn test_empirical_integrate_discrete_gaussian_tail() -> Fallible<()> {
    let scale = 10.;
    let tail = 20;
    test_gaussian_tail(scale, tail)
}

#[test]
// Ignored because this runs very slowly.
// Checks that test_empirical_integrate_discrete_gaussian_tail doesn't fail when run many times.
#[ignore]
pub fn test_empirical_integrate_discrete_gaussian_tail_multi_run() -> Fallible<()> {
    let scale = 10.;
    let tail = 20;

    let handles = (0..10)
        .map(|_| thread::spawn(move || (0..10).try_for_each(|_| test_gaussian_tail(scale, tail))))
        .collect::<Vec<JoinHandle<_>>>();

    handles
        .into_iter()
        .try_for_each(|h| h.join().expect("thread failed"))
}

#[test]
fn test_laplace_discrete_upper_bounded_by_continuous() -> Fallible<()> {
    for scale in [1., 10., 100., 1000., 10000., u32::MAX as f64] {
        for tail in [1, 10, 100, 1000, 10000, u32::MAX] {
            let alpha_discrete = conservative_discrete_laplacian_tail_to_alpha(
                RBig::try_from(scale)?,
                UBig::from(tail),
            )?;
            let alpha_continuous = conservative_continuous_laplacian_tail_to_alpha(
                RBig::try_from(scale)?,
                RBig::from(tail),
            )?;
            // The greatest differences in bounds comes from large point masses,
            // so differences between the bounds decrease as fewer large point masses are considered.
            // Thus difference decreases as scale and tail gets larger.
            println!(
                "scale: {scale: >10}, tail: {tail: >10}, difference {}",
                alpha_continuous - alpha_discrete
            );
            assert!(
                alpha_discrete <= alpha_continuous,
                "scale: {scale}, tail: {tail}"
            );

            // The same relationship holds for the discrete gaussian,
            // but in that case the conservative discrete bound is implemented via the conservative continuous bound,
            // so the test is not repeated.
        }
    }
    Ok(())
}

#[test]
fn test_tail_bounds_reject_invalid_arguments() {
    // zero scale previously panicked in rational division; negative scale
    // previously flipped the exponent sign and returned alpha > 1
    for scale in [rbig!(0), rbig!(-1)] {
        assert!(conservative_continuous_laplacian_tail_to_alpha(scale.clone(), rbig!(1)).is_err());
        assert!(conservative_discrete_laplacian_tail_to_alpha(scale.clone(), ubig!(1)).is_err());
        assert!(conservative_continuous_gaussian_tail_to_alpha(scale, rbig!(1)).is_err());
    }
    assert!(conservative_continuous_laplacian_tail_to_alpha(rbig!(1), rbig!(-1)).is_err());
}

/// Demonstrates why the original code rounded in the wrong direction
///
/// The function computes alpha = Pr[X > t] = exp(-t/s) / 2.
/// We need to over estimate -t/s so that exp(-t/s) is also overestimated.
/// Traditionally negative numerators would require the opposite, but within
/// the exponent the direction is not flipped for the bound.
///
/// The previous code rounded -t/s down which decreases alpha.
///
/// Inputs are chosen to make the effect testable:
/// t = 5121, s = 10 gives exponent -512.1, which is not representable in
/// f64, and near 512 one f64 ulp is 2^-43 (~1e-13), so the two cast
/// directions land measurably far apart:
///
///   alpha_old  = 1.98045884372127425e-223   <- BELOW the true value (bug)
///                              27425
///   alpha_true = 1.98045884372131905e-223   (inside the certified bracket)
///                              31905
///   alpha_new  = 1.98045884372149923e-223   <- above the true value (fixed)
///                              49923
#[test]
fn test_continuous_laplace_tail_rounding_direction() -> Fallible<()> {
    use crate::traits::InfDiv;

    let (scale, tail) = (rbig!(10), rbig!(5121));

    // the OLD chain, reproduced verbatim: exponent rounded DOWN before exp
    let alpha_old = f64::neg_inf_cast(-tail.clone() / scale.clone())?
        .inf_exp()?
        .inf_div(&2.0)?;

    // the FIXED function: exponent rounded UP before exp
    let alpha_new = conservative_continuous_laplacian_tail_to_alpha(scale, tail)?;

    // certified bracket of exp(-5121/10) / 2
    let truth = Interval::from_f64(-5121.0)
        .div(&Interval::from_f64(10.0))
        .exp()
        .div(&Interval::from_f64(2.0));
    assert_rounds_up(alpha_new, alpha_old, &truth);
    Ok(())
}

/// alpha = exp(-t/s) / (exp(1/s) + 1) must be over-estimated:
/// numerator rounds UP, denominator DOWN. t/s = 512.1 is inexact in f64.
#[test]
fn test_discrete_laplace_tail_rounding_direction() -> Fallible<()> {
    use crate::traits::{InfAdd, InfDiv};

    let (scale, tail) = (rbig!(10), ubig!(5121));

    // the same chain with every rounding direction mirrored
    let numer_bad = f64::neg_inf_cast(-RBig::from(tail.clone()) / scale.clone())?.neg_inf_exp()?;
    let denom_bad = f64::inf_cast(RBig::ONE / scale.clone())?
        .inf_exp()?
        .inf_add(&1.)?;
    let alpha_bad = numer_bad.neg_inf_div(&denom_bad)?;

    let alpha_impl = conservative_discrete_laplacian_tail_to_alpha(scale, tail)?;

    // certified bracket of exp(-5121/10) / (exp(1/10) + 1)
    let one = Interval::from_f64(1.0);
    let ten = Interval::from_f64(10.0);
    let truth = Interval::from_f64(-5121.0)
        .div(&ten)
        .exp()
        .div(&one.div(&ten).exp().add(&one));
    assert_rounds_up(alpha_impl, alpha_bad, &truth);
    Ok(())
}

// TODO: a rounding-direction test for the gaussian tail bounds needs an erfc reference bracket

/// The discrete gaussian tail delegates to the continuous bound (CKS20 Prop 25).
#[test]
fn test_discrete_gaussian_tail_delegation() -> Fallible<()> {
    let (scale, tail) = (rbig!(10), ubig!(13));

    let alpha_disc = conservative_discrete_gaussian_tail_to_alpha(scale.clone(), tail.clone())?;
    let alpha_cont = conservative_continuous_gaussian_tail_to_alpha(scale, RBig::from(tail))?;
    assert_eq!(alpha_disc, alpha_cont);
    Ok(())
}
