//! Lazy exact standard normal sampling, extracted from sampler-refactor (#2866).
//! Karney/DFW20 decomposition: a half discrete Gaussian integer plus an
//! accepted uniform fractional part, then an independent fair sign.
use dashu::{integer::UBig, rational::RBig, rbig};
use crate::error::Fallible;
use super::{
    psrn::{PartialUniform01, Uniform01RV},
    PartialSample, sample_bernoulli_exp, sample_geometric_exp_fast, sample_standard_bernoulli,
};

// Nonnegative integer k with mass proportional to exp(-k^2 / 2).
// The geometric proposal has mass proportional to exp(-k/2);
// accepting with exp(-(k-1/2)^2/2) gives the required law.
fn sample_half_standard_gaussian() -> Fallible<UBig> {
    loop {
        let k = sample_geometric_exp_fast(rbig!(1 / 2))?;
        let offset = RBig::from(k.clone()) - rbig!(1 / 2);
        if sample_bernoulli_exp(offset.pow(2) / rbig!(2))? {
            return Ok(k);
        }
    }
}

/// One Bernoulli trial with success probability `exp(-x)` for lazy `x`.
///
/// This is the generalized von Neumann decreasing-sequence test used by DFW20.
fn sample_bernoulli_exp_uniform(x: &mut PartialUniform01) -> Fallible<bool> {
    // First term: continue iff U_1 < x.
    let mut y = PartialSample::new(Uniform01RV);

    if !x.greater_than(&mut y)? {
        return Ok(true);
    }

    // We have seen one success, so the next failure returns false.
    let mut accept_on_failure = false;

    loop {
        let mut u = PartialSample::new(Uniform01RV);

        // Continue iff U_{n+1} < U_n.
        if !y.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        y = u;
        accept_on_failure = !accept_on_failure;
    }
}

/// Return whether `u < x / 2` for lazy uniforms `u` and `x`.
fn sample_bernoulli_u_lt_x2(u: &mut PartialUniform01, x: &mut PartialUniform01) -> Fallible<bool> {
    loop {
        if u.upper().unwrap() * rbig!(2) < x.lower().unwrap() {
            return Ok(true);
        }

        if u.lower().unwrap() * rbig!(2) > x.upper().unwrap() {
            return Ok(false);
        }

        if u.refinements() <= x.refinements().saturating_add(1) {
            u.refine()?;
        } else {
            x.refine()?;
        }
    }
}

/// One Bernoulli trial with success probability `exp(-x^2 / 2)`.
///
/// This is DFW20's `B_{e^{-xy}}` algorithm with parameters `(x/2, x)`.
fn sample_bernoulli_exp_half_x_squared(x: &mut PartialUniform01) -> Fallible<bool> {
    // First continuation event:
    //     u_1 < x / 2
    //     v_1 < x
    let mut w = PartialSample::new(Uniform01RV);

    if !sample_bernoulli_u_lt_x2(&mut w, x)? {
        return Ok(true);
    }

    let mut v = PartialSample::new(Uniform01RV);
    if !x.greater_than(&mut v)? {
        return Ok(true);
    }

    // One completed continuation, so next failure returns false.
    let mut accept_on_failure = false;

    loop {
        let mut u = PartialSample::new(Uniform01RV);

        // Later continuation events use the decreasing-uniform chain:
        //     u_{i+1} < u_i
        if !w.greater_than(&mut u)? {
            return Ok(accept_on_failure);
        }

        let mut v = PartialSample::new(Uniform01RV);
        if !x.greater_than(&mut v)? {
            return Ok(accept_on_failure);
        }

        w = u;
        accept_on_failure = !accept_on_failure;
    }
}

/// Accept the fractional part x with probability
///
///     exp(-x * (2k + x) / 2).
///
/// DFW20 factorizes this as
///
///     exp(-k x) * exp(-x^2 / 2).
fn accept_fraction(k: &UBig, x: &mut PartialUniform01) -> Fallible<bool> {
    let mut remaining = k.clone();

    while !remaining.is_zero() {
        if !sample_bernoulli_exp_uniform(x)? {
            return Ok(false);
        }
        remaining -= UBig::ONE;
    }

    sample_bernoulli_exp_half_x_squared(x)
}

pub(super) struct ExactStdNormal {
    negative: bool,
    k: RBig,
    x: PartialUniform01,
}

/// Sample an exact lazy standard normal in Karney form:
///
///     Z = S * (K + X).
pub(super) fn sample_exact_std_normal() -> Fallible<ExactStdNormal> {
    loop {
        let k = sample_half_standard_gaussian()?;
        let mut x = PartialSample::new(Uniform01RV);

        if !accept_fraction(&k, &mut x)? {
            continue;
        }

        let negative = sample_standard_bernoulli()?;
        let k = RBig::from(k);

        return Ok(ExactStdNormal { negative, k, x });
    }
}

impl ExactStdNormal {
    pub(super) fn bounds(&self) -> (RBig, RBig) {
        let lo = &self.k + self.x.lower().unwrap();
        let hi = &self.k + self.x.upper().unwrap();
        if self.negative { (-hi, -lo) } else { (lo, hi) }
    }

    pub(super) fn refine(&mut self) -> Fallible<()> {
        self.x.refine()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_normal_refines_same_dyadic_trace() -> Fallible<()> {
        let mut z = sample_exact_std_normal()?;
        let (lo, hi) = z.bounds();
        assert!(lo < hi && (hi.clone() - &lo) <= RBig::ONE);
        z.refine()?;
        let (new_lo, new_hi) = z.bounds();
        assert!(lo <= new_lo && new_lo < new_hi && new_hi <= hi);
        assert_eq!((hi - lo) / (new_hi - new_lo), RBig::from(UBig::ONE << 64usize));
        Ok(())
    }
}
