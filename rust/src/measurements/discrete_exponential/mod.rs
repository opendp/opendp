use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::InfDifferenceDistance,
    traits::{samplers::SampleUniform, CheckNull, Float, InfCast, Number, RoundCast},
};

#[cfg(feature = "ffi")]
mod ffi;

#[bootstrap(features("contrib", "floating-point"))]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
/// 
/// # Arguments
/// * `temperature` - Higher temperatures are more private.
/// 
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
/// * `QO` - Output Distance Type.
pub fn make_base_discrete_exponential<TIA, QO>(
    temperature: QO,
) -> Fallible<
    Measurement<VectorDomain<AllDomain<TIA>>, usize, InfDifferenceDistance<TIA>, MaxDivergence<QO>>,
>
where
    TIA: Clone + CheckNull + Number,
    QO: 'static + InfCast<TIA> + RoundCast<TIA> + Float + SampleUniform,
{
    if temperature.is_sign_negative() || temperature.is_zero() {
        return fallible!(MakeMeasurement, "temperature must be positive");
    }
    Ok(Measurement::new(
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<TIA>| {
            arg.iter()
                .cloned()
                .map(|v| QO::round_cast(v).map(|v| v / temperature))
                // enumerate before sampling so that indexes are inside the result
                .enumerate()
                // gumbel samples are porous
                .map(|(i, llik)| {
                    let llik = llik?;
                    QO::sample_standard_uniform(false).map(|u| (i, llik - u.ln().neg().ln()))
                })
                // retrieve the highest noisy likelihood pair
                .try_fold((arg.len(), QO::neg_infinity()), |acc: (usize, QO), res| {
                    res.map(|v| if acc.1 > v.1 { acc } else { v })
                })
                // only return the index
                .map(|v| v.0)
        }),
        InfDifferenceDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            let d_in = QO::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            // d_out >= d_in / temperature
            d_in.inf_div(&temperature)
        }),
    ))
}

#[cfg(feature = "floating-point")]
#[cfg(test)]
pub mod test_exponential {
    use super::*;

    #[test]
    fn test_exponential() -> Fallible<()> {
        let de = make_base_discrete_exponential(1.)?;
        let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
        println!("{:?}", release);

        Ok(())
    }
}
