use std::convert::TryFrom;

use crate::{
    core::{Function, Postprocessor},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    traits::Float, trans::make_postprocess,
};

use super::check_parameters;

#[cfg(feature="ffi")]
mod ffi;

pub fn make_postprocess_sized_proportion_ci<TA>(
    strat_sizes: Vec<usize>,
    sample_sizes: Vec<usize>,
    scale: TA,
) -> Fallible<Postprocessor<VectorDomain<AllDomain<TA>>, VectorDomain<AllDomain<TA>>>>
where
    TA: Float,
{
    check_parameters(&strat_sizes, &sample_sizes)?;

    let (_0, _1) = (From::from(0), From::from(1));

    // cast sizes to TA
    let sample_sizes = (sample_sizes.into_iter())
        .map(TA::round_cast)
        .collect::<Fallible<Vec<TA>>>()?;
    let strat_sizes = (strat_sizes.into_iter())
        .map(TA::round_cast)
        .collect::<Fallible<Vec<TA>>>()?;

    // compute weights
    let strat_size = strat_sizes.iter().copied().sum();
    let weights: Vec<TA> = strat_sizes.iter().map(|&v| v / strat_size).collect();

    make_postprocess(
        VectorDomain::new_all(),
        VectorDomain::new_all(),
        Function::new(move |sample_sums: &Vec<TA>| {
            // use the noisy sample sums and public sample sizes
            let sample_means: Vec<TA> = (sample_sums.iter())
                .zip(sample_sizes.iter())
                .map(|(&strat_sum, &sample_size)| (strat_sum / sample_size).min(_1).max(_0))
                .collect();

            // dot product of weights and means
            let mean = (weights.iter())
                .zip(sample_means.iter())
                .map(|(&w, &sample_mean)| w * sample_mean)
                .sum();

            // dot product between weights^2 and variances
            let variance = (strat_sizes.iter())
                .zip(sample_sizes.iter())
                .zip(sample_means.iter())
                .map(|((&N, &n), &p)| {
                    // (N - n)/n (p(1-p) + σ/n^2) / (n-1) + σ/n^2
                    (N - n) / N * (p * (_1 - p) + scale / n.powi(2)) / (n - _1) + scale / n.powi(2)
                })
                .zip(weights.iter())
                .map(|(strat_var, &w)| w.powi(2) * strat_var)
                .sum();

            vec![mean, variance]
        }),
    )
}

pub fn make_postprocess_proportion_ci<TA>(
    strat_sizes: Vec<usize>,
    sum_scale: TA,
    size_scale: TA,
) -> Fallible<Postprocessor<VectorDomain<VectorDomain<AllDomain<TA>>>, VectorDomain<AllDomain<TA>>>>
where
    TA: Float,
{
    if strat_sizes.iter().any(|&s| s == 0) {
        return fallible!(MakeTransformation, "partitions must be non-empty");
    }

    if strat_sizes.len() == 0 {
        return fallible!(MakeTransformation, "must have at least one partition");
    }

    let (_0, _1, _2) = (From::from(0), From::from(1), From::from(2));

    // cast sizes to TA
    let strat_sizes = (strat_sizes.into_iter())
        .map(TA::round_cast)
        .collect::<Fallible<Vec<TA>>>()?;

    // compute weights
    let strat_size = strat_sizes.iter().copied().sum();
    let weights: Vec<TA> = strat_sizes.iter().map(|&v| v / strat_size).collect();

    make_postprocess(
        VectorDomain::new(VectorDomain::new_all()),
        VectorDomain::new_all(),
        Function::new_fallible(move |arg: &Vec<Vec<TA>>| {
            let [mut sample_sums, mut sample_sizes] = <[Vec<TA>; 2]>::try_from(arg.clone())
                .map_err(|_| err!(FailedFunction, "expected an input of [sums, counts]"))?;

            sample_sums.iter_mut().for_each(|v| *v = v.max(_0));
            sample_sizes.iter_mut().for_each(|v| *v = v.max(_2));

            let sample_means = (sample_sums.iter())
                .zip(sample_sizes.iter())
                .map(|(&strat_sum, &sample_size)| (strat_sum / sample_size).min(_1).max(_0))
                .collect::<Vec<TA>>();

            let mean = (weights.iter())
                .zip(sample_means.iter())
                .map(|(&w, &strat_mean)| w * strat_mean)
                .sum();

            let variance = (strat_sizes.iter())
                .zip(sample_sizes.iter())
                .zip(sample_means.iter())
                .map(|((&N, &n), &p)| {
                    // (N - n)/(n-1) (p(1-p) + (σ_sum + σ_size p^2) / n) / n
                    (N - n) / (N - _1) * (p * (_1 - p) + (sum_scale + p.powi(2) * size_scale) / n)
                        / n
                })
                .zip(weights.iter())
                .map(|(strat_var, &w)| w.powi(2) * strat_var)
                .sum();

            Ok(vec![mean, variance])
        }),
    )
}