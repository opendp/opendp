use crate::{
    combinators::select_private_candidate::bounds::{
        new_conditional_rdp_curve, new_negative_binomial_rdp_curve, new_poisson_rdp_curve,
        solve_nb_x_from_mean,
    },
    core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, RenyiDivergence},
    traits::{
        CastInternalRational, InfAdd, InfMul, NextFloat,
        samplers::{
            logarithmic::sample_logarithmic_exp,
            negative_binomial::sample_truncated_negative_binomial_rational,
            poisson::sample_poisson,
        },
    },
};
use dashu::{integer::UBig, rational::RBig};
use std::fmt::Debug;

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

mod bounds;

/// Repetition law for repeated private candidate selection.
///
/// `Poisson` uses a Poisson repetition count with mean `mean`.
/// `NegativeBinomial { eta }` uses the Papernot--Steinke truncated negative-binomial family,
/// with `eta = 1` corresponding to the geometric case and `eta = 0` corresponding to the
/// logarithmic case.
///
/// # Proof Definition
/// `Repetitions` is a public specification of the repetition family.
/// `Repetitions::Poisson` corresponds to the Poisson repetition law analyzed in
/// Theorem 6 of Papernot--Steinke (2021).
/// `Repetitions::NegativeBinomial { eta }` corresponds to the truncated negative-binomial
/// family of Definition 1, including the special cases:
/// * `eta = 1`: geometric
/// * `eta = 0`: logarithmic
#[derive(Clone, Debug)]
pub enum Repetitions {
    Poisson,
    NegativeBinomial { eta: f64 },
}

impl Repetitions {
    #[allow(non_upper_case_globals)]
    pub const Geometric: Self = Self::NegativeBinomial { eta: 1.0 };

    #[allow(non_upper_case_globals)]
    pub const Logarithmic: Self = Self::NegativeBinomial { eta: 0.0 };

    /// Resolve the public repetition specification into the internal sampling/accounting
    /// parameterization.
    ///
    /// For Poisson, the mean is used directly.
    /// For the NB family, this solves for the internal `x` parameter and rounds `x`
    /// upward conservatively before use.
    ///
    /// # Proof Definition
    /// For any finite `mean` and any valid `self`,
    /// either returns `Err(e)` if the requested repetition family is invalid,
    /// or returns `Ok(out)` where:
    /// * `Poisson` resolves to a Poisson repetition law with mean `mean`;
    /// * `NegativeBinomial { eta }` resolves to the truncated negative-binomial family
    ///   of Definition 1 with shape `eta` and an internal parameter `x` chosen
    ///   conservatively for the implemented sampler.
    ///
    /// # Paper Reference
    /// Papernot--Steinke (2021), Definition 1; Theorem 6.
    fn resolve(self, mean: f64) -> Fallible<ResolvedRepetitions> {
        match self {
            Self::Poisson => {
                if mean < 0.0 {
                    return fallible!(MakeMeasurement, "Poisson mean must be nonnegative");
                }

                Ok(ResolvedRepetitions::Poisson { mean })
            }

            Self::NegativeBinomial { eta } => {
                if mean <= 1.0 {
                    return fallible!(
                        MakeMeasurement,
                        "negative-binomial and logarithmic means must be strictly greater than 1"
                    );
                }
                if !eta.is_finite() {
                    return fallible!(MakeMeasurement, "eta must be finite");
                }
                if eta < 0.0 {
                    return fallible!(MakeMeasurement, "eta must be nonnegative");
                }

                let x = solve_nb_x_from_mean(mean, eta)?.next_up_();
                Ok(ResolvedRepetitions::NegativeBinomial { eta, x })
            }
        }
    }
}

impl Default for Repetitions {
    fn default() -> Self {
        Self::Geometric
    }
}

/// Select a private candidate from repeated private candidates.
///
/// `measurement` should make releases in the form of `(score, candidate)`.
///
/// Supported parameter combinations:
///
/// * `measurement` under `MaxDivergence` and `threshold=Some(_)`:
///   Liu and Talwar (2019), with `distribution=Repetitions::Geometric` only,
///   and privacy cost `2 * measurement.map(d_in)`.
/// * `measurement` under `MaxDivergence` and `threshold=None`:
///   Papernot and Steinke (2021) Corollary 3,
///   with `distribution` in the negative-binomial family, and privacy cost
///   `(2 + eta) * measurement.map(d_in)`.
/// * `measurement` under `RenyiDivergence` and `threshold=Some(_)`:
///   Appendix C Corollary 16 of Papernot and Steinke (2021), with
///   `distribution=Repetitions::Geometric` only.
/// * `measurement` under `RenyiDivergence` and `threshold=None`:
///   Papernot and Steinke (2021), with `distribution=Repetitions::Poisson` (Theorem 6) or any
///   negative-binomial family member (Theorem 2).
///
/// Unsupported combinations raise a construction-time error.
///
/// # Arguments
/// * `measurement` - A measurement that releases a 2-tuple of (score, candidate)
/// * `mean` - The requested mean number of repetitions
/// * `threshold` - If set, release the first candidate whose score is at least this threshold. Otherwise, return the best candidate among the sampled repetitions.
/// * `distribution` - The repetition distribution
pub fn make_select_private_candidate<
    DI: 'static + Domain,
    MI: 'static + Metric,
    MO: 'static + PrivateCandidateMeasure,
    TO: 'static + Debug,
>(
    measurement: Measurement<DI, MI, MO, (f64, TO)>,
    mean: f64,
    threshold: Option<f64>,
    distribution: Repetitions,
) -> Fallible<Measurement<DI, MI, MO, Option<(f64, TO)>>>
where
    (DI, MI): MetricSpace,
{
    if !mean.is_finite() {
        return fallible!(MakeMeasurement, "mean must be finite");
    }
    if let Some(threshold) = threshold {
        if !threshold.is_finite() {
            return fallible!(MakeMeasurement, "threshold must be finite");
        }
    }

    let repetitions = distribution.resolve(mean)?;

    MO::validate(threshold.is_some(), &repetitions)?;

    let function = measurement.function.clone();
    let privacy_map = MO::new_privacy_map(
        measurement.privacy_map.clone(),
        threshold.is_some(),
        mean,
        &repetitions,
    )?;

    Measurement::new(
        measurement.input_domain.clone(),
        measurement.input_metric.clone(),
        measurement.output_measure.clone(),
        Function::new_fallible(move |arg| {
            let mut remaining = repetitions.sample()?;
            let mut best = None;

            while remaining > UBig::ZERO {
                let next = function.eval(arg)?;

                if let Some(threshold) = threshold {
                    if next.0 >= threshold {
                        return Ok(Some(next));
                    }
                } else {
                    best = choose(best, next);
                }

                remaining -= UBig::ONE;
            }

            Ok(best)
        }),
        privacy_map,
    )
}

/// Internal resolved repetition family used by the implementation.
///
/// # Proof Definition
/// `ResolvedRepetitions` stores the exact internal parameters needed by the sampler:
/// * `Poisson { mean }` samples from `Poisson(mean)`;
/// * `NegativeBinomial { eta, x }` samples from the logarithmic or truncated
///   negative-binomial family with `gamma = 1 - exp(-x)`.
#[derive(Clone, Debug)]
pub enum ResolvedRepetitions {
    Poisson { mean: f64 },
    NegativeBinomial { eta: f64, x: f64 },
}

impl ResolvedRepetitions {
    /// Sample exactly from the resolved repetition law.
    ///
    /// # Proof Definition
    /// For any valid resolved repetitions:
    /// * `Poisson { mean }` returns a draw from `Poisson(mean)` on `ℕ₀`;
    /// * `NegativeBinomial { eta: 0, x }` returns a draw from the logarithmic
    ///   distribution `D_{0,γ}` of Definition 1 with `γ = 1 - exp(-x)`;
    /// * `NegativeBinomial { eta, x }` with `eta > 0` returns a draw from
    ///   `D_{η,γ}` of Definition 1 with `γ = 1 - exp(-x)`.
    ///
    /// # Paper Reference
    /// Definition 1; Theorem 6.
    fn sample(&self) -> Fallible<UBig> {
        match self {
            Self::Poisson { mean } => sample_poisson(RBig::try_from(*mean)?),
            Self::NegativeBinomial { eta, x } => {
                let x_r = RBig::try_from(*x)?;
                if *eta == 0.0 {
                    sample_logarithmic_exp(x_r)
                } else {
                    sample_truncated_negative_binomial_rational(eta.into_rational()?, x_r)
                }
            }
        }
    }

    fn is_geometric(&self) -> bool {
        matches!(self, Self::NegativeBinomial { eta, .. } if *eta == 1.0)
    }
}

/// Keep the highest-scoring non-NaN candidate seen so far.
///
/// # Proof Definition
/// For any `best` and `next`,
/// returns:
/// * `best` unchanged if `next.0` is `NaN`,
/// * otherwise the higher-scoring of `best` and `next`,
///   breaking ties in favor of `next`.
///
/// # Note
/// The paper assumes a total order on the output space and returns the maximum
/// under that order.
fn choose<T>(best: Option<(f64, T)>, next: (f64, T)) -> Option<(f64, T)> {
    if next.0.is_nan() {
        return best;
    }

    let Some(best) = best else { return Some(next) };
    Some(if best.0 > next.0 { best } else { next })
}

/// Internal extension trait for privacy measures supported by private candidate selection.
///
/// # Proof Definition
/// `validate` accepts exactly the combinations of selection mode and repetition family
/// supported by the implemented privacy results.
/// `new_privacy_map` lifts the base privacy map of the underlying candidate mechanism to the
/// repeated-selection privacy map.
pub trait PrivateCandidateMeasure: Measure {
    fn validate(has_threshold: bool, repetitions: &ResolvedRepetitions) -> Fallible<()>;

    fn new_privacy_map<MI: 'static + Metric>(
        base_map: PrivacyMap<MI, Self>,
        has_threshold: bool,
        mean: f64,
        repetitions: &ResolvedRepetitions,
    ) -> Fallible<PrivacyMap<MI, Self>>;
}

impl PrivateCandidateMeasure for MaxDivergence {
    fn validate(has_threshold: bool, repetitions: &ResolvedRepetitions) -> Fallible<()> {
        if has_threshold && !repetitions.is_geometric() {
            return fallible!(
                MakeMeasurement,
                "thresholded MaxDivergence selection requires geometric repetitions"
            );
        }
        if matches!(repetitions, ResolvedRepetitions::Poisson { .. }) {
            return fallible!(
                MakeMeasurement,
                "Poisson selection is not supported under MaxDivergence"
            );
        }
        Ok(())
    }

    fn new_privacy_map<MI: 'static + Metric>(
        base_map: PrivacyMap<MI, Self>,
        has_threshold: bool,
        _mean: f64,
        repetitions: &ResolvedRepetitions,
    ) -> Fallible<PrivacyMap<MI, Self>> {
        let factor = if has_threshold {
            2.0
        } else {
            match repetitions {
                ResolvedRepetitions::NegativeBinomial { eta, .. } => 2.0f64.inf_add(eta)?,
                ResolvedRepetitions::Poisson { .. } => unreachable!("validated above"),
            }
        };

        Ok(PrivacyMap::new_fallible(move |d_in| {
            base_map.eval(d_in)?.inf_mul(&factor)
        }))
    }
}

impl PrivateCandidateMeasure for RenyiDivergence {
    fn validate(has_threshold: bool, repetitions: &ResolvedRepetitions) -> Fallible<()> {
        if has_threshold && !repetitions.is_geometric() {
            return fallible!(
                MakeMeasurement,
                "thresholded Renyi-Divergence selection requires geometric repetitions"
            );
        }
        Ok(())
    }

    fn new_privacy_map<MI: 'static + Metric>(
        base_map: PrivacyMap<MI, Self>,
        has_threshold: bool,
        mean: f64,
        repetitions: &ResolvedRepetitions,
    ) -> Fallible<PrivacyMap<MI, Self>> {
        let repetitions = repetitions.clone();

        Ok(PrivacyMap::new_fallible(move |d_in| {
            let base_curve = base_map.eval(d_in)?;
            Ok(match (&repetitions, has_threshold) {
                (ResolvedRepetitions::NegativeBinomial { x, .. }, true) => {
                    new_conditional_rdp_curve(base_curve, *x)
                }
                (ResolvedRepetitions::NegativeBinomial { eta, x }, false) => {
                    new_negative_binomial_rdp_curve(base_curve, *eta, *x, mean)
                }
                (ResolvedRepetitions::Poisson { .. }, false) => {
                    new_poisson_rdp_curve(base_curve, mean)
                }
                (ResolvedRepetitions::Poisson { .. }, true) => unreachable!("validated above"),
            })
        }))
    }
}
