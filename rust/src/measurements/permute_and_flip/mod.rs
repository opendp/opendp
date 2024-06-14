use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measurements::Optimize,
    measures::MaxDivergence,
    metrics::LInfDistance,
    traits::samplers::{sample_bernoulli_exp, CastInternalRational, SampleUniformIntBelow},
    traits::{DistanceConstant, Float, Number},
};
use dashu::base::Sign;
use dashu::rational::RBig;

fn exact_fisher_yates(n: usize, trials_per_coin: Option<usize>) -> Fallible<Vec<usize>> {
    let mut permutation: Vec<usize> = (0..n).collect();

    for i in (1..n).rev() {
        let j = usize::sample_uniform_int_below(i, trials_per_coin)?;
        permutation.swap(i, j);
    }

    Ok(permutation)
}

/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score via permute and flip.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable VectorDomain.
/// * `input_metric` - Metric on the input domain. Must be LInfDistance
/// * `scale` - Higher scales are more private.
/// * `optimize` - Indicate whether to privately return the "Max" or "Min"
///
/// # Generics
/// * `TIA` - Atom Input Type. Type of each element in the score vector.
/// * `QO` - Output Distance Type.
pub fn make_report_noisy_max_permute_and_flip<TIA, QO>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    scale: QO,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MaxDivergence<QO>>>
where
    TIA: Number + CastInternalRational,
    QO: CastInternalRational + DistanceConstant<TIA> + Float,
{
    if input_domain.element_domain.nullable() {
        return fallible!(MakeMeasurement, "input domain must be non-nullable");
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale must not be negative");
    }

    let scale_frac: RBig = scale.into_rational()?;
    let sgn: Sign = match optimize {
        Optimize::Max => Sign::Positive,
        Optimize::Min => Sign::Negative,
    };

    Measurement::new(
        input_domain,
        Function::new_fallible(move |arg: &Vec<TIA>| {
            let permutation = exact_fisher_yates(arg.len(), None)?;
            let rational_elements = arg
                .iter()
                .map(|x| ((*x).into_rational().map(|x| x * sgn)))
                .collect::<Fallible<Vec<RBig>>>()?;

            // get argmax of the rational elements
            let argmax = rational_elements
                .iter()
                .enumerate()
                .max_by_key(|&(_, value)| value)
                .map(|(index, _)| index)
                .ok_or(err!(FailedFunction, "there must be at least one candidate"))?;

            let max_element = rational_elements
                .get(argmax)
                .ok_or(err!(FailedFunction, "there must be at least one candidate"))?;

            // iterate over the permutations and throw a coin for each
            for i in permutation {
                let candidate = rational_elements
                    .get(i)
                    .ok_or(err!(FailedFunction, "Wrong indexing"))?;

                let coin_bias_scale = max_element - candidate;
                let coin_flip = sample_bernoulli_exp(coin_bias_scale / &scale_frac)?;
                if coin_flip {
                    return Ok(i);
                }
            }
            Ok(argmax)
        }),
        input_metric.clone(),
        MaxDivergence::default(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(*d_in)?;

            // convert data type to QO
            let d_in = QO::inf_cast(d_in)?;

            if d_in.is_sign_negative() {
                return fallible!(InvalidDistance, "sensitivity must be non-negative");
            }

            if scale.is_zero() {
                return Ok(QO::infinity());
            }

            let rational_two =
                QO::inf_cast(TIA::from(2).ok_or(err!(FailedCast, "Failed to cast constant."))?)?;

            // d_out >= (2 * d_in) / scale
            (d_in * rational_two).inf_div(&scale)
        }),
    )
}

#[cfg(feature = "floating-point")]
#[cfg(test)]
mod test;
