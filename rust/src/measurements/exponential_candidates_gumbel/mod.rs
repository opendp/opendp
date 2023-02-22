use num::Float;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::InfDifferenceDistance,
    traits::{CheckNull, InfDiv, InfMul, samplers::SampleUniform},
};

pub fn make_base_exponential_candidates_gumbel<TI>(
    temperature: TI,
    constant_time: bool,
) -> Fallible<
    Measurement<
        VectorDomain<AllDomain<TI>>,
        AllDomain<usize>,
        InfDifferenceDistance<TI>,
        MaxDivergence<TI>,
    >,
>
where
    TI: 'static + CheckNull + Float + SampleUniform + InfMul + InfDiv,
{
    Ok(Measurement::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new_fallible(move |arg: &Vec<TI>| {
            arg.iter()
                .copied()
                .map(|v| v / temperature)
                // enumerate before sampling so that indexes are inside the result
                .enumerate()
                // gumbel samples are porous
                .map(|(i, llik)| {
                    TI::sample_standard_uniform(constant_time)
                        .map(|u| (i, llik - u.ln().neg().ln()))
                })
                // retrieve the highest noisy likelihood pair
                .try_fold((arg.len(), TI::neg_infinity()), |acc: (usize, TI), res| {
                    res.map(|v| if acc.1 > v.1 { acc } else { v })
                })
                // only return the index
                .map(|v| v.0)
        }),
        InfDifferenceDistance::default(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TI| {
            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }
            // d_out >= d_in / temperature
            d_in.inf_div(&temperature)
        }),
    ))
}

#[cfg(test)]
pub mod test_exponential {
    use super::*;

    #[test]
    fn test_exponential() -> Fallible<()> {
        let de = make_base_exponential_candidates_gumbel(1., false)?;
        let release = de.invoke(&vec![1., 2., 3., 2., 1.])?;
        println!("{:?}", release);

        Ok(())
    }
}
