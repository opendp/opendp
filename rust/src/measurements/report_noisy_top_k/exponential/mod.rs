use core::f64;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::{
        CastInternalRational, InfCast, InfDiv, InfMul, Number,
        samplers::{Shuffle, sample_bernoulli_exp},
    },
};
use dashu::base::Sign;
use dashu::rational::RBig;
use num::Zero;

#[cfg(test)]
mod test;

/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score via permute and flip.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance.
/// * `k` - Number of times to run the algorithm.
/// * `scale` - Higher scales are more private.
/// * `negate` - Set to true to return min.
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
pub fn make_permute_and_flip<TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    k: usize,
    scale: f64,
    negate: bool,
) -> Fallible<
    Measurement<VectorDomain<AtomDomain<TIA>>, Vec<usize>, LInfDistance<TIA>, MaxDivergence>,
>
where
    TIA: Number + CastInternalRational,
    f64: InfCast<TIA> + InfCast<usize>,
{
    if input_domain.element_domain.nan() {
        return fallible!(MakeMeasurement, "input domain elements must be non-nan");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let size = input_domain
        .size
        .ok_or_else(|| err!(MakeTransformation, "input_domain member size must be known"))?;

    if size < k {
        return fallible!(
            MakeTransformation,
            "k ({k}) must not be greater than the number of candidates ({size})"
        );
    }

    let scale_frac: RBig = scale.into_rational()?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            permute_and_flip(arg, k, scale_frac.clone(), negate)
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = f64::inf_cast(input_metric.range_distance(*d_in)?)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity ({d_in}) must be non-negative");
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_out >= d_in / scale * k
            d_in.inf_div(&scale)?.inf_mul(&f64::inf_cast(k)?)
        }),
    )
}

pub(crate) fn permute_and_flip<TIA: Clone + CastInternalRational>(
    x: &[TIA],
    k: usize,
    scale: RBig,
    negate: bool,
) -> Fallible<Vec<usize>> {
    let sign = Sign::from(negate);
    let rational_elements = x
        .iter()
        .cloned()
        .map(|x_i| x_i.into_rational().map(|x_i| x_i * sign))
        .collect::<Fallible<Vec<RBig>>>()?;

    let mut permutation: Vec<usize> = (0..x.len()).collect();
    (0..k)
        .map(|_| {
            permutation.shuffle()?;

            // get argmax of the rational elements
            let max_candidate = rational_elements
                .iter()
                .max()
                .ok_or(err!(FailedFunction, "there must be at least one candidate"))?;

            // iterate over the permutations and throw a coin for each
            for i in &permutation {
                let candidate = &rational_elements[*i];
                if sample_bernoulli_exp((max_candidate - candidate) / &scale)? {
                    return Ok(permutation.remove(*i));
                }
            }
            fallible!(
                FailedFunction,
                "Enumerating over all the candidates will always terminate."
            )
        })
        .collect()
}
