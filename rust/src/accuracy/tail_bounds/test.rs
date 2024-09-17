use crate::{
    domains::AtomDomain,
    measurements::{make_gaussian, make_laplace},
    metrics::AbsoluteDistance,
    traits::{samplers::SampleUniformIntBelow, ExactIntCast},
};

use super::*;

fn test_laplace_tail(tail: u32, theoretical_alpha: f64, label: &str) -> Fallible<()> {
    let scale = 1.;

    println!("alpha: {}", theoretical_alpha);
    let m_dlap = make_laplace(
        AtomDomain::<i8>::default(),
        AbsoluteDistance::default(),
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
    let scale = 1.;
    let tail = 2;
    let alpha = conservative_discrete_laplacian_tail_to_alpha(scale, tail)?;
    test_laplace_tail(tail, alpha, "Discrete Laplace")
}

#[test]
pub fn test_empirical_integrate_discrete_laplace_tail_random() -> Fallible<()> {
    let scale = 1.;
    let tail = 1 + u32::sample_uniform_int_below(10, None)?;
    let alpha = conservative_discrete_laplacian_tail_to_alpha(scale, tail)?;
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
    let alpha_upper = conservative_discrete_gaussian_tail_to_alpha(scale, tail)?;
    let alpha_avg = discrete_gaussian_tail_to_alpha(scale, tail)?;

    let m_dnorm = make_gaussian::<_, _, f64>(
        AtomDomain::<i8>::default(),
        AbsoluteDistance::default(),
        scale,
        None,
    )?;
    let n = 50_000;
    let alpha_emp = (0..n)
        .filter(|_| m_dnorm.invoke(&0).unwrap().clamp(-127, 127) > tail as i8)
        .count() as f64
        / n as f64;

    println!("Discrete Gaussian significance levels/alpha");
    println!("Theoretical (upper): {:?}", alpha_upper);
    println!("Theoretical (avg):   {:?}", alpha_avg);
    println!("Empirical:           {:?}", alpha_emp);

    // this test has a small likelihood of failing
    // I'm observing that alpha_upper can be loose by a factor of ~10,
    // as it uses the tail bound of the continuous gaussian
    assert!(alpha_emp < alpha_upper);
    Ok(())
}

#[test]
pub fn test_empirical_integrate_discrete_gaussian_tail_fixed() -> Fallible<()> {
    let scale = 10.;
    let tail = 30;
    test_gaussian_tail(scale, tail)
}
