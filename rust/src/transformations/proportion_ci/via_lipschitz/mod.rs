use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AllDomain, VectorDomain},
    error::{ExplainUnwrap, Fallible},
    metrics::{AbsoluteDistance, L1Distance},
    traits::Float,
};

use super::check_parameters;

#[cfg(feature="ffi")]
mod ffi;

pub fn make_lipschitz_sized_proportion_ci_mean<TA: Float>(
    strat_sizes: Vec<usize>,
    sample_sizes: Vec<usize>,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TA>>,
        AllDomain<TA>,
        L1Distance<TA>,
        AbsoluteDistance<TA>,
    >,
> {
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

    let stability_constant = (weights.iter())
        .zip(sample_sizes.iter())
        .map(|(&w, &n)| w * n)
        .reduce(|l, r| l.max(r))
        .unwrap_assert("there is always at least one partition");

    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |sample_sums: &Vec<TA>| {
            (sample_sums.iter())
                .zip(sample_sizes.iter())
                .map(|(&s, &n)| (s / n).min(_1).max(_0))
                .zip(weights.iter())
                .map(|(mean, &w)| mean * w)
                .sum()
        }),
        L1Distance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &TA| d_in.inf_mul(&stability_constant)),
    ))
}

pub fn make_lipschitz_sized_proportion_ci_variance<TA: Float>(
    strat_sizes: Vec<usize>,
    sample_sizes: Vec<usize>,
    mean_scale: TA,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TA>>,
        AllDomain<TA>,
        L1Distance<TA>,
        AbsoluteDistance<TA>,
    >,
> {
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

    // let function constant c_i = (N_i - n_i) / (N_i * (n_i - 1)) * w^2
    //     where N_i is strat size i and n_i is sample_size i
    let function_constants = (strat_sizes.iter())
        .zip(sample_sizes.iter())
        .zip(weights.iter())
        .map(|((&N, &n), &w)| (N - n) / (N * (n - _1)) * w.powi(2))
        .collect::<Vec<TA>>();

    let stability_constant = (strat_sizes.iter())
        .zip(sample_sizes.iter())
        .zip(weights.iter())
        .map(|((&N, &n), &w)| (w / n).powi(2) * (N - n) / N)
        .reduce(|l, r| l.max(r))
        .unwrap_assert("there is always at least one partition");

    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new(move |sample_sums: &Vec<TA>| {
            (sample_sums.iter())
                .zip(sample_sizes.iter())
                .map(|(&s, &n)| (s / n).min(_1).max(_0))
                .zip(function_constants.iter())
                .map(|(p, &c)| p * (_1 - p) * c)
                .sum::<TA>()
                + mean_scale.powi(2)
        }),
        L1Distance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_fallible(move |d_in: &TA| d_in.inf_mul(&stability_constant)),
    ))
}
