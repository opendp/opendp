#[cfg(feature = "ffi")]
mod ffi;

use std::ops::Neg;

use num::{Float as _, One, Zero};

use crate::core::{Domain, Function, Measurement, PrivacyMap, SensitivityMetric};
use crate::domains::{AllDomain, VectorDomain};
use crate::error::*;
use crate::measures::MaxDivergence;
use crate::metrics::{AbsoluteDistance, L1Distance};
use crate::traits::samplers::SampleDiscreteLaplace;
use crate::traits::{Float, InfAdd, InfDiv, InfLn, InfLog2, InfPow, RoundCast, InfMul, InfSub};

pub trait LaplaceDomain: Domain {
    type Metric: SensitivityMetric<Distance = Self::Atom>;
    type Atom: Float + SampleDiscreteLaplace;
    fn new() -> Self;
    fn noise_function(scale: Self::Atom, gran_pow: i32) -> Function<Self, Self>;
}

impl<T> LaplaceDomain for AllDomain<T>
where
    T: Float + SampleDiscreteLaplace,
{
    type Metric = AbsoluteDistance<T>;
    type Atom = Self::Carrier;

    fn new() -> Self {
        AllDomain::new()
    }
    fn noise_function(scale: Self::Carrier, gran_pow: i32) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            Self::Carrier::sample_discrete_laplace(*arg, scale, gran_pow)
        })
    }
}

impl<T> LaplaceDomain for VectorDomain<AllDomain<T>>
where
    T: Float + SampleDiscreteLaplace,
{
    type Metric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self {
        VectorDomain::new_all()
    }
    fn noise_function(scale: T, gran_pow: i32) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            arg.iter()
                .map(|v| T::sample_discrete_laplace(*v, scale, gran_pow))
                .collect()
        })
    }
}

pub fn make_base_laplace<D>(
    scale: D::Atom,
) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
where
    D: LaplaceDomain,
    i32: RoundCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let _2 = D::Atom::one() + D::Atom::one();

    // Want to find the largest `scale 2^-k` such that coin flip probability `p` is still nonzero
    // Utility is maximized when k is made smaller.
    // Want to compute the smallest feasible k in a way that makes k conservatively larger.

    // Now solve for the smallest k (granularity power) such that p > 0.
    // We know p = 1 - α, where α = exp(-1/(scale 2^-k)).
    // First choose the largest α such that p != 0
    let max_alpha = {
        let min_p = _2.powi(-44);
        println!("min_p {:?}", min_p);    
        D::Atom::one().inf_sub(&min_p)?
    };

    println!("max alpha {:?}", max_alpha);

    // Since α = exp(-1/(scale 2^-k)), solve for k = gran_pow:
    //        => k = log2(-scale ln(α))
    // Implement with conservative rounding towards larger k for a small loss in utility
    let gran_pow = max_alpha
        .neg_inf_ln()?
        .neg()
        .inf_mul(&scale)?
        .inf_log2()?
        .ceil();
    println!("gran pow {:?}", gran_pow);

    // We will round values to the nearest multiple of 2^-k
    let relaxation = {
        let _2 = D::Atom::one() + D::Atom::one();
        _2.inf_pow(&gran_pow)?
    };

    Ok(Measurement::new(
        D::new(),
        D::new(),
        // this cast is always exact because of the ceil
        D::noise_function(scale, i32::round_cast(gran_pow)?),
        D::Metric::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &D::Atom| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            if scale.is_zero() {
                return Ok(D::Atom::infinity());
            }
            // (d_in + r) / scale
            d_in.inf_add(&relaxation)?.inf_div(&scale)
        }),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{metrics::SymmetricDistance, trans::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (make_sized_bounded_mean::<SymmetricDistance, _>(3, (10.0, 12.0))?
            >> make_base_laplace(1.0)?)?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())
    }

    #[test]
    fn test_make_base_laplace() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(1.0)?;
        let _ret = measurement.invoke(&0.0)?;
        println!("1e0 map: {:?}", measurement.map(&1.)?);
        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_shifted() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(1.0)?;
        println!("1e0 ret: {:?}", measurement.invoke(&10.0)?);
        println!("1e0 map: {:?}", measurement.map(&1.)?);
        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_zero_scale() -> Fallible<()> {
        let measurement = make_base_laplace::<AllDomain<_>>(0.0)?;
        assert_eq!(1.0, measurement.invoke(&1.0)?);
        Ok(())
    }

    #[test]
    fn test_make_base_laplace_small_scale() -> Fallible<()> {
        let meas_1e20 = make_base_laplace::<AllDomain<_>>(1e-20)?;
        println!("1e20 res: {:?}", meas_1e20.invoke(&0.)?);
        println!("1e20 map: {:?}", meas_1e20.map(&1.)?);

        let meas_1e100 = make_base_laplace::<AllDomain<_>>(1e-100)?;
        println!("1e100 res: {:?}", meas_1e100.invoke(&0.)?);
        println!("1e100 map: {:?}", meas_1e100.map(&1.)?);

        Ok(())
    }

    #[test]
    fn test_make_base_laplace_large_scale() -> Fallible<()> {
        let meas_1e20 = make_base_laplace::<AllDomain<_>>(1e20)?;
        println!("1e20 res: {:?}", meas_1e20.invoke(&0.)?);
        println!("1e20 map: {:?}", meas_1e20.map(&1.)?);

        let meas_1e100 = make_base_laplace::<AllDomain<_>>(1e100)?;
        println!("1e100 res: {:?}", meas_1e100.invoke(&0.)?);
        println!("1e100 map: {:?}", meas_1e100.map(&1.)?);

        Ok(())
    }

    #[test]
    fn test_make_base_laplace_vec() -> Fallible<()> {
        let measurement = make_base_laplace::<VectorDomain<_>>(1.0)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }
}
