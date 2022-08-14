use crate::{
    core::{Function, StabilityMap, Transformation},
    domains::{AllDomain, VectorDomain},
    error::{ExplainUnwrap, Fallible},
    metrics::{AbsoluteDistance, L1Distance},
    traits::{Float, Integer, RoundCast, InfCast},
};

use super::check_parameters;

#[cfg(feature = "ffi")]
mod ffi;

pub fn make_lipschitz_sized_proportion_ci_mean<TIA: Integer, TOA: Float>(
    strat_sizes: Vec<usize>,
    sample_sizes: Vec<usize>,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TIA>>,
        AllDomain<TOA>,
        L1Distance<TIA>,
        AbsoluteDistance<TOA>,
    >,
>
where
    TOA: RoundCast<TIA> + InfCast<TIA>,
{
    check_parameters(&strat_sizes, &sample_sizes)?;

    let (_0, _1) = (From::from(0), From::from(1));

    // cast sizes to TOA
    let sample_sizes = (sample_sizes.into_iter())
        .map(TOA::round_cast)
        .collect::<Fallible<Vec<TOA>>>()?;
    let strat_sizes = (strat_sizes.into_iter())
        .map(TOA::round_cast)
        .collect::<Fallible<Vec<TOA>>>()?;

    // compute weights
    let strat_size = strat_sizes.iter().copied().sum();
    let weights: Vec<TOA> = strat_sizes.iter().map(|&v| v / strat_size).collect();

    let stability_constant = (weights.iter())
        .zip(sample_sizes.iter())
        .map(|(&w, &n)| w * n)
        .reduce(|l, r| l.max(r))
        .unwrap_assert("there is always at least one partition");

    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new_fallible(move |sample_sums: &Vec<TIA>| {
            // convert sample_sums to TOA
            let sample_sums = sample_sums
                .iter()
                .cloned()
                .map(TOA::round_cast)
                .collect::<Fallible<Vec<TOA>>>()?;
            Ok((sample_sums.into_iter())
                .zip(sample_sizes.iter())
                .map(|(s, &n)| (s / n).min(_1).max(_0))
                .zip(weights.iter())
                .map(|(mean, &w)| mean * w)
                .sum())
        }),
        L1Distance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(stability_constant),
    ))
}

pub fn make_lipschitz_sized_proportion_ci_variance<TIA: Integer, TOA: Float>(
    strat_sizes: Vec<usize>,
    sample_sizes: Vec<usize>,
    mean_scale: TOA,
) -> Fallible<
    Transformation<
        VectorDomain<AllDomain<TIA>>,
        AllDomain<TOA>,
        L1Distance<TIA>,
        AbsoluteDistance<TOA>,
    >,
>
where TOA: RoundCast<TIA> + InfCast<TIA>, {
    check_parameters(&strat_sizes, &sample_sizes)?;

    let (_0, _1) = (From::from(0), From::from(1));

    // cast sizes to TA
    let sample_sizes = (sample_sizes.into_iter())
        .map(TOA::round_cast)
        .collect::<Fallible<Vec<TOA>>>()?;
    let strat_sizes = (strat_sizes.into_iter())
        .map(TOA::round_cast)
        .collect::<Fallible<Vec<TOA>>>()?;

    // compute weights
    let strat_size = strat_sizes.iter().copied().sum();
    let weights: Vec<TOA> = strat_sizes.iter().map(|&v| v / strat_size).collect();

    // let function constant c_i = (N_i - n_i) / (N_i * (n_i - 1)) * w^2
    //     where N_i is strat size i and n_i is sample_size i
    let function_constants = (strat_sizes.iter())
        .zip(sample_sizes.iter())
        .zip(weights.iter())
        .map(|((&N, &n), &w)| (N - n) / (N * (n - _1)) * w.powi(2))
        .collect::<Vec<TOA>>();

    let stability_constant = (strat_sizes.iter())
        .zip(sample_sizes.iter())
        .zip(weights.iter())
        .map(|((&N, &n), &w)| w.powi(2) * (N - n) / N / (n - _1 / n))
        .reduce(|l, r| l.max(r))
        .unwrap_assert("there is always at least one partition");

    Ok(Transformation::new(
        VectorDomain::new_all(),
        AllDomain::new(),
        Function::new_fallible(move |sample_sums: &Vec<TIA>| {
            // convert sample_sums to TOA
            let sample_sums = sample_sums
                .iter()
                .cloned()
                .map(TOA::round_cast)
                .collect::<Fallible<Vec<TOA>>>()?;
            Ok((sample_sums.into_iter())
                .zip(sample_sizes.iter())
                .map(|(s, &n)| (s / n).min(_1).max(_0))
                .zip(function_constants.iter())
                .map(|(p, &c)| p * (_1 - p) * c)
                .sum::<TOA>()
                + mean_scale.powi(2))
        }),
        L1Distance::default(),
        AbsoluteDistance::default(),
        StabilityMap::new_from_constant(stability_constant),
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    // there are integration tests in python
    #[test]
    fn test_lipschitz_sized_proportion_ci_mean() -> Fallible<()> {
        let strat_sizes = vec![100usize; 10];
        let sample_sizes = vec![10usize; 10];
        let trans = make_lipschitz_sized_proportion_ci_mean::<i32, f64>(strat_sizes, sample_sizes)?;

        println!("invoke {:?}", trans.invoke(&vec![5; 10])?);
        println!("map {:?}", trans.map(&1)?);
        Ok(())
    }

    #[test]
    fn test_lipschitz_sized_proportion_ci_variance() -> Fallible<()> {
        let strat_sizes = vec![100usize; 10];
        let sample_sizes = vec![10usize; 10];
        let trans = make_lipschitz_sized_proportion_ci_variance::<i32, f64>(strat_sizes, sample_sizes, 1.)?;

        println!("invoke {:?}", trans.invoke(&vec![5; 10])?);
        println!("map {:?}", trans.map(&1)?);
        Ok(())
    }
}
