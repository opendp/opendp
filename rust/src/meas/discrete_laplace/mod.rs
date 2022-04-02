#[cfg(feature = "ffi")]
mod ffi;

use num::{Float, One, Zero};

use crate::core::{Domain, Measurement, SensitivityMetric};
use crate::dist::{AbsoluteDistance, L1Distance, MaxDivergence};
use crate::dom::{AllDomain, VectorDomain};
use crate::error::*;
use crate::meas::make_base_geometric;
use crate::samplers::{
    SampleLaplace, SampleUniform, SampleUniformExponent,
};
use crate::traits::{
    CheckNull, InfAdd, InfCast, InfDiv, InfExp, InfMul, InfSub, RoundCast, TotalOrd,
};
use crate::trans::{make_integerize, make_lipschitz_extension, LipschitzDomain};

type IntegerAtom = i64;

pub trait LaplaceDomain: LipschitzDomain {
    type Metric: SensitivityMetric<Distance = Self::Atom> + Default;
    type Atom;
    type IntegerDomain: Domain;
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
    type IntegerDomain = AllDomain<IntegerAtom>;
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
    type IntegerDomain = VectorDomain<AllDomain<IntegerAtom>>;
}

pub fn make_base_discrete_laplace<D>(
    scale: D::Atom,
    bounds: Option<(D::Atom, D::Atom)>,
    granularity: Option<D::Atom>,
) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
where
    D: LaplaceDomain + Default,
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
    let bounds: Option<(i64, i64)> = bounds
        .map(|(l, u): (_, _)| {
            Result::<_, Error>::Ok((i64::round_cast(l * c)?, i64::round_cast(u * c)?))
        })
        .transpose()?;

    let scale: D::Atom = scale * c;

    make_lipschitz_extension::<D, D::Metric>(c)?
        >> make_integerize::<D, D::IntegerDomain, D::Metric>()?
        >> make_base_geometric::<D::IntegerDomain, D::Atom>(scale, bounds)?
        >> make_integerize::<D::IntegerDomain, D, D::Metric>()?
        >> make_lipschitz_extension::<D, D::Metric>(c.recip())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dom::VectorDomain, trans::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (make_sized_bounded_mean(3, (10.0, 12.0))?
            >> make_base_discrete_laplace(1.0, None, None)?)?;
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
