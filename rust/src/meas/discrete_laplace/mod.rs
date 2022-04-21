// #[cfg(feature = "ffi")]
// mod ffi;

mod traits;
pub use traits::*;

use num::Float;

use crate::core::Measurement;
use crate::dist::{AbsoluteDistance, MaxDivergence};
use crate::error::*;
use crate::meas::make_base_geometric;
use crate::traits::{InfAdd, InfCast, RoundCast};
use crate::trans::{make_lipschitz_cast, make_lipschitz_mul, GreatestDifference, SameMetric};


pub fn make_base_discrete_laplace<D, I>(
    scale: D::Atom,
    bounds: Option<(D::Atom, D::Atom)>,
    granularity: Option<D::Atom>,
) -> Fallible<Measurement<D, D, D::Metric, MaxDivergence<D::Atom>>>
where
    D: DiscreteLaplaceDomain<I>,
    I: 'static
        + InfCast<D::Atom>
        + RoundCast<D::Atom>
        + Clone
        + Ord
        + GreatestDifference<D::Atom>
        + InfAdd,
    // metrics match, but associated distance types may vary
    (D::Metric, D::IntegerMetric): SameMetric<D::Metric, D::IntegerMetric>,
{
    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    // unwrap granularity or fill with default
    let granularity: D::Atom = granularity.unwrap_or_else(|| scale * D::Atom::GRAN);
    // derive the constant to convert to int space
    let c: D::Atom = (-granularity.log2().ceil()).exp2();
    // translate bounds to int space
    let bounds: Option<(I, I)> = bounds
        .map(|(l, u): (_, _)| {
            Result::<_, Error>::Ok((I::round_cast(l * c)?, I::round_cast(u * c)?))
        })
        .transpose()?;

    let scale: D::Atom = scale * c;

    make_lipschitz_mul::<D, D::Metric>(c)?
        >> make_lipschitz_cast::<D, D::IntegerDomain, D::Metric, D::IntegerMetric>()?
        >> make_base_geometric::<D::IntegerDomain, D::Atom>(scale, bounds)?
        // these metrics are ignored. Arbitrarily chosen to minimize the number of necessary trait bounds
        >> make_lipschitz_cast::<D::IntegerDomain, D, AbsoluteDistance<D::Atom>, AbsoluteDistance<D::Atom>>()?
        >> make_lipschitz_mul::<D, AbsoluteDistance<D::Atom>>(c.recip())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dom::{VectorDomain, AllDomain}, trans::make_sized_bounded_mean};

    #[test]
    fn test_chain_laplace() -> Fallible<()> {
        let chain = (make_sized_bounded_mean(3, (10.0, 12.0))?
            >> make_base_discrete_laplace::<_, i64>(1.0f64, None, None)?)?;
        let _ret = chain.invoke(&vec![10.0, 11.0, 12.0])?;
        Ok(())
    }

    #[test]
    fn test_make_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<AllDomain<_>, i64>(1.0, None, None)?;
        let _ret = measurement.invoke(&0.0)?;

        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }

    #[test]
    fn test_make_vector_laplace_mechanism() -> Fallible<()> {
        let measurement = make_base_discrete_laplace::<VectorDomain<_>, i64>(1.0, None, None)?;
        let arg = vec![1.0, 2.0, 3.0];
        let _ret = measurement.invoke(&arg)?;

        assert!(measurement.check(&1., &1.0001)?);
        Ok(())
    }
}
