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
