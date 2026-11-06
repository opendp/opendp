#[cfg(feature = "ffi")]
mod ffi;

use crate::core::{Measure, PrivacyMap};
use crate::error::Fallible;
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::ChangeOneDistance;
use crate::traits::{ExactIntCast, InfDiv, InfExpM1, InfLn1P, InfMul};

/// Analyze fixed-size sampling without replacement under substitution adjacency.
///
/// Implementations return a privacy map over population distances. They may use
/// group privacy to extend a neighboring guarantee, or use a direct analysis.
///
/// # Proof Definition
/// Given a privacy map for the complete sample-level analysis under
/// `ChangeOneDistance`, returns either `Err(e)` or `Ok(map)`, where
/// `map(d)` bounds the privacy loss of applying that analysis after uniformly
/// sampling `sample_size` records without replacement from a fixed-size
/// population of `population_size` records, at population distance `d`.
/// The sample is drawn independently of the data, and sample membership is
/// not disclosed.
///
/// # Citations
/// * [BBG18 Privacy Amplification by Subsampling: Tight Analyses via Couplings and Divergences, Theorem 9](https://papers.nips.cc/paper/2018/file/3b5020bb891119b9f5130f1fea9bd773-Paper.pdf)
/// * [DR14 The Algorithmic Foundations of Differential Privacy, Theorem 2.2](https://www.cis.upenn.edu/~aaroth/Papers/privacybook.pdf)
pub trait AmplifyWithoutReplacement: Measure {
    fn amplify_map(
        &self,
        privacy_map: PrivacyMap<ChangeOneDistance, Self>,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<PrivacyMap<ChangeOneDistance, Self>>
    where
        Self: Sized;
}

fn amplify_epsilon(epsilon: f64, sampling_rate: f64) -> Fallible<f64> {
    // Reject invalid budgets, including negative zero. Genuine infinity is a
    // valid (vacuous) privacy guarantee, distinct from finite overflow.
    if epsilon.is_nan() || epsilon.is_sign_negative() {
        return fallible!(FailedMap, "epsilon ({epsilon:?}) must be non-negative");
    }
    if epsilon == f64::INFINITY {
        return Ok(f64::INFINITY);
    }

    // BBG18 Theorem 9 gives ln(1 + q * (exp(epsilon) - 1)).
    // Directed rounding of each operation yields a conservative upper bound.
    epsilon.inf_exp_m1()?.inf_mul(&sampling_rate)?.inf_ln_1p()
}

impl AmplifyWithoutReplacement for MaxDivergence {
    fn amplify_map(
        &self,
        privacy_map: PrivacyMap<ChangeOneDistance, Self>,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<PrivacyMap<ChangeOneDistance, Self>> {
        // By BBG18 Theorem 9, amplify_epsilon bounds the privacy loss for
        // one population substitution. By group privacy (DR14 Theorem 2.2),
        // d_in substitutions cost at most d_in * epsilon.
        // Upward-rounded multiplication preserves the upper bound.
        let sampling_rate =
            f64::exact_int_cast(sample_size)?.inf_div(&f64::exact_int_cast(population_size)?)?;
        let epsilon = amplify_epsilon(privacy_map.eval(&1)?, sampling_rate)?;

        Ok(PrivacyMap::new_fallible(move |d_in: &u32| {
            if *d_in == 0 {
                return Ok(0.0);
            }
            if epsilon == f64::INFINITY {
                return Ok(f64::INFINITY);
            }
            epsilon.inf_mul(&f64::exact_int_cast(*d_in)?)
        }))
    }
}

/// Extend an adjacent approximate-DP guarantee to a group of substitutions.
fn group_privacy_approx((epsilon, delta): (f64, f64), group_size: u32) -> Fallible<(f64, f64)> {
    // Chaining approximate-DP inequalities for d substitutions gives
    // (d * epsilon, delta * sum_{j=0}^{d-1} exp(j * epsilon)).
    // For positive epsilon, the sum is expm1(d * epsilon) / expm1(epsilon).
    // The numerator is rounded up, the denominator down, and the quotient and
    // final product up. Arithmetic errors propagate; successfully computed
    // delta bounds are capped at one.
    if group_size == 0 {
        return Ok((0.0, 0.0));
    }
    if group_size == 1 {
        return Ok((epsilon, delta));
    }

    // Infinity is an actual input guarantee, not finite arithmetic overflow.
    if epsilon == f64::INFINITY {
        return Ok((f64::INFINITY, if delta == 0.0 { 0.0 } else { 1.0 }));
    }

    let epsilon_group = epsilon.inf_mul(&f64::exact_int_cast(group_size)?)?;
    if delta == 0.0 {
        return Ok((epsilon_group, 0.0));
    }

    let group_size = f64::exact_int_cast(group_size)?;
    let delta_group = if epsilon == 0.0 {
        delta.inf_mul(&group_size)?
    } else {
        let numerator = epsilon.inf_mul(&group_size)?.inf_exp_m1()?;
        let denominator = epsilon.neg_inf_exp_m1()?;
        delta.inf_mul(&numerator.inf_div(&denominator)?)?
    };
    Ok((epsilon_group, delta_group.min(1.0)))
}

impl AmplifyWithoutReplacement for Approximate<MaxDivergence> {
    fn amplify_map(
        &self,
        privacy_map: PrivacyMap<ChangeOneDistance, Self>,
        population_size: usize,
        sample_size: usize,
    ) -> Fallible<PrivacyMap<ChangeOneDistance, Self>> {
        // BBG18 Theorem 9 bounds adjacent population privacy by
        // (ln(1 + q * (exp(epsilon) - 1)), q * delta), for q = sample_size /
        // population_size. The arithmetic is rounded upward.
        // group_privacy_approx then extends this bound to all distances.
        let sampling_rate =
            f64::exact_int_cast(sample_size)?.inf_div(&f64::exact_int_cast(population_size)?)?;
        let (epsilon, delta) = privacy_map.eval(&1)?;
        if delta.is_nan() || delta.is_sign_negative() || delta > 1.0 {
            return fallible!(FailedMap, "delta ({delta:?}) must be between zero and one");
        }
        let adjacent_loss = (
            amplify_epsilon(epsilon, sampling_rate)?,
            delta.inf_mul(&sampling_rate)?,
        );

        Ok(PrivacyMap::new_fallible(move |d_in: &u32| {
            group_privacy_approx(adjacent_loss, *d_in)
        }))
    }
}

/// Analyze privacy amplification by sampling without replacement.
///
/// The input and population metrics are both [`ChangeOneDistance`].
/// The map describes the complete sample-level analysis for a fixed-size sample
/// of `sample_size` records from a population of `population_size` records.
///
/// # Why honest-but-curious?
/// The privacy guarantees are only valid if the input dataset is a simple
/// random sample without replacement from a population with `population_size`
/// records, drawn independently of the data. Sample membership must not be
/// disclosed, and the supplied privacy map must account for all releases
/// involving the sample.
pub fn amplify_without_replacement<MO>(
    input_metric: ChangeOneDistance,
    output_measure: MO,
    privacy_map: PrivacyMap<ChangeOneDistance, MO>,
    population_size: usize,
    sample_size: usize,
) -> Fallible<PrivacyMap<ChangeOneDistance, MO>>
where
    MO: 'static + AmplifyWithoutReplacement,
    MO::Distance: 'static,
{
    let _ = input_metric;
    // The sample-size check establishes 1 <= sample_size <= population_size.
    // The output map satisfies the population-level privacy guarantee by the
    // proof definition of AmplifyWithoutReplacement::amplify_map.
    if sample_size == 0 || sample_size > population_size {
        return fallible!(
            FailedMap,
            "sample size ({:?}) must be between 1 and population size ({:?})",
            sample_size,
            population_size
        );
    }

    output_measure.amplify_map(privacy_map, population_size, sample_size)
}

#[cfg(all(test, feature = "partials"))]
mod test;
