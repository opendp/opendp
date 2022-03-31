#[cfg(feature = "ffi")]
mod ffi;

use num::{Float, One, Zero};

use crate::core::{Domain, Function, Measurement, PrivacyRelation, SensitivityMetric};
use crate::dist::{AbsoluteDistance, MaxDivergence, L1Distance};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::samplers::{
    SampleLaplace, SampleTwoSidedGeometric, SampleUniform, SampleUniformExponent,
};
use crate::traits::{
    CheckNull, InfAdd, InfCast, InfDiv, InfExp, InfMul, InfSub, RoundCast, TotalOrd,
};

pub trait LaplaceDomain: Domain {
    type Metric: SensitivityMetric<Distance = Self::Atom> + Default;
    type Atom;
    fn new() -> Self;
    fn noise_function(k: Self::Atom, scale: Self::Atom, bounds: Option<(i64, i64)>) -> Function<Self, Self>;
}

impl<T> LaplaceDomain for AllDomain<T>
where
    T: 'static
        + Float
        + CheckNull
        + RoundCast<i64>
        + InfExp
        + InfSub
        + Copy
        + One
        + Zero
        + PartialOrd
        + SampleUniformExponent
        + InfDiv
        + InfAdd
        + SampleUniform,
    T::Bits: PartialOrd,
    i64: RoundCast<T>,
{
    type Metric = AbsoluteDistance<T>;
    type Atom = Self::Carrier;

    fn new() -> Self {
        AllDomain::new()
    }
    fn noise_function(l: Self::Atom, scale: Self::Atom, bounds: Option<(i64, i64)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            let int_value = i64::round_cast(arg.clone() * l)?;
            let pri_value = i64::sample_two_sided_geometric(int_value, scale, bounds)?;
            Self::Carrier::round_cast(pri_value)
        })
    }
}

impl<T> LaplaceDomain for VectorDomain<AllDomain<T>>
where
    T: 'static
        + Float
        + CheckNull
        + RoundCast<i64>
        + InfExp
        + InfSub
        + Copy
        + One
        + Zero
        + PartialOrd
        + SampleUniformExponent
        + InfDiv
        + InfAdd
        + SampleUniform,
    T::Bits: PartialOrd,
    i64: RoundCast<T>,
{
    type Metric = L1Distance<T>;
    type Atom = T;

    fn new() -> Self {
        VectorDomain::new_all()
    }
    fn noise_function(l: Self::Atom, scale: Self::Atom, bounds: Option<(i64, i64)>) -> Function<Self, Self> {
        Function::new_fallible(move |arg: &Self::Carrier| {
            arg.iter().map(|v| {
                let int_value = i64::round_cast(v.clone() * l)?;
                let pri_value = i64::sample_two_sided_geometric(int_value, scale, bounds)?;
                Self::Atom::round_cast(pri_value)
            }).collect()
        })
    }
}

pub fn make_base_discrete_laplace<D>(
    scale: D::Atom,
    bounds: Option<(D::Atom, D::Atom)>,
    granularity: Option<D::Atom>,
) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
where
    D: LaplaceDomain,
    D::Atom: 'static + Clone + SampleLaplace + Float + InfCast<f64> + CheckNull + TotalOrd + InfMul,
    i64: RoundCast<D::Atom>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    // unwrap granularity or fill with default
    let granularity: D::Atom = granularity.unwrap_or_else(|| scale.powi(-8));
    // derive the constant to convert to int space
    let c: D::Atom = (-granularity.log2().ceil()).exp2();
    // translate bounds to int space
    let bounds: Option<(i64, i64)> = bounds.map(|(l, u): (_, _)| 
        Result::<_, Error>::Ok((i64::round_cast(l * c)?, i64::round_cast(u * c)?))).transpose()?;

    let scale: D::Atom = scale * c;

    Ok(Measurement::new(
        D::new(),
        D::new(),
        D::noise_function(c, scale.clone(), bounds),
        D::Metric::default(),
        MaxDivergence::default(),
        PrivacyRelation::new_all(
            move |d_in: &D::Atom, d_out: &D::Atom| {
                if d_in.is_sign_negative() {
                    return fallible!(InvalidDistance, "sensitivity must be non-negative");
                }
                if d_out.is_sign_negative() {
                    return fallible!(InvalidDistance, "epsilon must be non-negative");
                }
                // d_out * scale >= d_in
                Ok(d_out.neg_inf_mul(&scale)? >= d_in.clone())
            },
            Some(move |d_out: &D::Atom| d_out.neg_inf_mul(&scale)),
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dom::VectorDomain, trans::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain =
            (make_sized_bounded_mean(3, (10.0, 12.0))? >> make_base_discrete_laplace(1.0, None, None)?)?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())
    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<AllDomain<_>>(1.0, None, None)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<VectorDomain<_>>(1.0, None, None)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.)?);
        Ok(())
    }
}
