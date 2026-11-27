use std::ops::{Add, Div, Mul, Neg, Sub};

use dashu::{base::Signed, rational::RBig};
use opendp_derive::{bootstrap, proven};

use crate::core::{Measure, Metric, MetricSpace};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily};
use crate::{
    core::{Domain, Measurement, PrivacyMap},
    error::Fallible,
    measures::{MaxDivergence, MultiDP, PrivacyGuarantee, curves::renyidp::rdp_from_pureDP},
    metrics::L1Distance,
    traits::{InfCast, SInterval},
};

#[cfg(test)]
mod test;

#[cfg(feature = "ffi")]
pub(crate) mod ffi;

#[bootstrap(
    features("contrib"),
    arguments(k(default = b"null")),
    generics(DI(suppress), MI(suppress), MO(default = "MaxDivergence"))
)]
/// Make a Measurement that adds noise from the Laplace(`scale`) distribution to the input.
///
/// Valid inputs for `input_domain` and `input_metric` are:
///
/// | `input_domain`                  | input type   | `input_metric`         |
/// | ------------------------------- | ------------ | ---------------------- |
/// | `atom_domain(T)` (default)      | `T`          | `absolute_distance(T)` |
/// | `vector_domain(atom_domain(T))` | `Vec<T>`     | `l1_distance(T)`       |
///
/// Internally, all sampling is done using the discrete Laplace distribution.
///
/// # Citations
/// * [GRS12 Universally Utility-Maximizing Privacy Mechanisms](https://theory.stanford.edu/~tim/papers/priv.pdf)
/// * [CKS20 The Discrete Gaussian for Differential Privacy](https://arxiv.org/pdf/2004.00010.pdf#subsection.5.2)
///
/// # Arguments
/// * `input_domain` - Domain of the data type to be released.
/// * `input_metric` - Metric of the data type to be released.
/// * `scale` - Noise scale parameter for the Laplace distribution. `scale` == standard_deviation / sqrt(2).
/// * `k` - The noise granularity in terms of 2^k, only valid for domains over floats.
///
/// # Generics
/// * `DI` - Domain of the data type to be released. Valid values are `VectorDomain<AtomDomain<T>>` or `AtomDomain<T>`
/// * `MI` - Metric used to measure distance between members of the input domain.
/// * `MO` - Measure used to quantify privacy loss. Valid values are `MaxDivergence` and `MultiDP`.
pub fn make_laplace<DI: Domain, MI: Metric, MO: Measure>(
    input_domain: DI,
    input_metric: MI,
    scale: f64,
    k: Option<i32>,
) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>
where
    DiscreteLaplace: MakeNoise<DI, MI, MO>,
    (DI, MI): MetricSpace,
{
    DiscreteLaplace { scale, k }.make_noise((input_domain, input_metric))
}

pub struct DiscreteLaplace {
    pub scale: f64,
    pub k: Option<i32>,
}

/// Laplace mechanism
#[proven(proof_path = "measurements/noise/distribution/laplace/MakeNoise_for_DiscreteLaplace.tex")]
impl<DI: NoiseDomain, MI: Metric, MO: 'static + Measure> MakeNoise<DI, MI, MO> for DiscreteLaplace
where
    (DI, MI): MetricSpace,
    DI::Atom: Nature,
    <DI::Atom as Nature>::RV<1>: MakeNoise<DI, MI, MO>,
{
    fn make_noise(self, input_space: (DI, MI)) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space)
    }
}

#[proven(
    proof_path = "measurements/noise/distribution/laplace/NoisePrivacyMap_for_ZExpFamily1.tex"
)]
impl NoisePrivacyMap<L1Distance<RBig>, MaxDivergence> for ZExpFamily<1> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L1Distance<RBig>,
        _output_measure: &MaxDivergence,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, MaxDivergence>> {
        let ZExpFamily { scale } = self.clone();
        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
        }
        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_negative() {
                return fallible!(FailedMap, "sensitivity ({}) must be positive", d_in);
            }

            if d_in.is_zero() {
                return Ok(0.0);
            }

            if scale.is_zero() {
                return Ok(f64::INFINITY);
            }

            // d_in / scale
            f64::inf_cast(d_in / scale.clone())
        }))
    }
}
#[proven(
    proof_path = "measurements/noise/distribution/laplace/NoisePrivacyMap_for_ZExpFamily1_MultiDP.tex"
)]
impl NoisePrivacyMap<L1Distance<RBig>, MultiDP> for ZExpFamily<1> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L1Distance<RBig>,
        _output_measure: &MultiDP,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, MultiDP>> {
        let ZExpFamily { scale } = self.clone();

        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({scale}) must not be negative");
        }

        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_negative() {
                return fallible!(FailedMap, "sensitivity ({d_in}) must be non-negative");
            }

            if d_in.is_zero() {
                return PrivacyGuarantee::new()
                    .with_approx_dp(0.0, 0.0)?
                    .with_zCDP(0.0, 0.0)?
                    .with_renyiDP(|orders| Ok(vec![0.0; orders.len()]), 0.0);
            }

            if scale.is_zero() {
                return Ok(PrivacyGuarantee::new());
            }

            let epsilon_exact = d_in / scale.clone();
            let epsilon = f64::inf_cast(epsilon_exact.clone())?;

            if !epsilon.is_finite() {
                return fallible!(FailedMap, "epsilon ({epsilon}) must be finite for MultiDP");
            }

            // Fall back to the pure-DP bound if interval evaluation fails.
            let rho = match zcdp_discrete_laplace(&epsilon_exact, d_in, &scale) {
                Ok(rho) => rho,
                Err(_) => epsilon,
            };
            let sensitivity = d_in.clone();
            let scale_for_rdp = scale.clone();

            // Trivial pure-DP representation.
            PrivacyGuarantee::new()
                .with_approx_dp(epsilon, 0.0)?
                // Tight zCDP representation from Harrison-Manurangsi.
                .with_zCDP(rho, 0.0)?
                // Exact discrete-Laplace RDP curve, with pure-DP RDP fallback if
                // numerical evaluation fails.
                .with_renyiDP(
                    move |orders| {
                        // Prepare the order-independent terms once per batch.
                        let exact_rdp = rdp_discrete_laplace(&sensitivity, &scale_for_rdp).ok();
                        orders
                            .iter()
                            .map(|&alpha| {
                                // Validate alpha before the fallback so invalid orders
                                // cannot be hidden by the pure-DP bound.
                                check_renyi_order(alpha)?;
                                if let Some(bound) =
                                    exact_rdp.as_ref().and_then(|rdp| rdp(alpha).ok())
                                {
                                    return Ok(bound);
                                }
                                rdp_from_pureDP(alpha, epsilon)
                            })
                            .collect()
                    },
                    0.0,
                )
        }))
    }
}

fn zcdp_discrete_laplace(epsilon: &RBig, sensitivity: &RBig, scale: &RBig) -> Fallible<f64> {
    if epsilon.is_zero() || sensitivity.is_zero() {
        return Ok(0.0);
    }

    let a = RBig::ONE / scale.clone();
    let epsilon_upper = f64::inf_cast(epsilon.clone())?;
    let a_upper = f64::inf_cast(a.clone())?;
    if !a_upper.is_finite() || a_upper > 700.0 {
        return Ok(epsilon_upper);
    }

    let epsilon_i = SInterval::from_rational(epsilon)?;
    let sensitivity_i = SInterval::from_rational(sensitivity)?;
    let a_i = SInterval::from_rational(&a)?;

    let one_minus_exp_neg_eps =
        SInterval::point(0.0)?.sub(SInterval::from_rational(&(-epsilon.clone()))?.exp_m1()?)?;
    let sinh_a = interval_sinh(a_i)?;
    let correction = one_minus_exp_neg_eps.div(sensitivity_i.mul(sinh_a)?)?;

    let rho = epsilon_i
        .mul(SInterval::point(1.0)?.sub(correction)?)?
        .clamp(0.0, epsilon_upper)?
        .upper_f64()?;

    if rho.is_nan() || rho < 0.0 {
        return fallible!(
            FailedMap,
            "computed zCDP rho ({rho}) must be non-negative and not NaN"
        );
    }

    Ok(rho)
}

fn rdp_discrete_laplace(
    sensitivity: &RBig,
    scale: &RBig,
) -> Fallible<impl Fn(f64) -> Fallible<f64>> {
    let a = RBig::ONE / scale.clone();
    let a_upper = f64::inf_cast(a.clone())?;
    if !a_upper.is_finite() || a_upper > 700.0 {
        return fallible!(FailedMap, "exact discrete-Laplace RDP numerics overflowed");
    }
    let a_i = SInterval::from_rational(&a)?;
    let d_i = SInterval::from_rational(sensitivity)?;
    let one = SInterval::point(1.0)?;
    let exp_a = a_i.clone().exp()?;
    let log_expm1_a = a_i.clone().exp_m1()?.ln()?;
    let log_tanh = log_expm1_a
        .clone()
        .sub(exp_a.clone().add(one.clone())?.ln()?)?;

    Ok(move |alpha| {
        check_renyi_order(alpha)?;
        let alpha_i = SInterval::point(alpha)?;
        let two = SInterval::point(2.0)?;

        // Proposition 15 / Lemma 16 form:
        //
        // D_alpha = 1 / (alpha - 1) *
        //   log[tanh(a/2) * (term1 + term2 + term3)]
        //
        // where a = 1 / scale and d = sensitivity.
        let log_t1 = a_i
            .clone()
            .mul(alpha_i.clone())?
            .mul(d_i.clone())?
            .neg()
            .sub(log_expm1_a.clone())?;

        // Factor the same exponential from the numerator and denominator.
        // Their ratio is exp(a * (alpha - 1) * d) *
        // (1 - exp(-a * (2 * alpha - 1) * (d + 1))) /
        // (1 - exp(-a * (2 * alpha - 1))).
        let gap = a_i
            .clone()
            .mul(two.mul(alpha_i.clone())?.sub(one.clone())?)?;
        let numerator = gap.clone().mul(d_i.clone().add(one.clone())?)?;
        let log_ratio = (-(-numerator).exp_m1()?)
            .ln()?
            .sub((-(-gap).exp_m1()?).ln()?)?;
        let log_t2 = a_i
            .clone()
            .mul(alpha_i.clone().sub(one.clone())?)?
            .mul(d_i.clone())?
            .add(log_ratio)?;

        let log_t3 = a_i
            .clone()
            .mul(alpha_i.clone().sub(one.clone())?)?
            .mul(d_i.clone())?
            .sub(log_expm1_a.clone())?;

        let log_sum = log_sum_exp3(log_t1, log_t2, log_t3)?;
        let log_moment = log_tanh.clone().add(log_sum)?;

        let epsilon = log_moment.div(alpha_i.sub(one.clone())?)?.upper_f64()?;

        if epsilon.is_nan() || epsilon < 0.0 {
            return fallible!(
                FailedMap,
                "computed RDP epsilon ({epsilon}) must be non-negative and not NaN"
            );
        }

        Ok(epsilon)
    })
}

fn log_sum_exp3(a: SInterval, b: SInterval, c: SInterval) -> Fallible<SInterval> {
    log_sum_exp_many([a, b, c])
}

fn log_sum_exp_many<const N: usize>(values: [SInterval; N]) -> Fallible<SInterval> {
    let mut pivot = f64::NEG_INFINITY;
    for value in &values {
        pivot = pivot.max(value.upper_f64()?);
    }
    let pivot_i = SInterval::point(pivot)?;

    let sum = values
        .into_iter()
        .try_fold(SInterval::point(0.0)?, |acc, value| {
            acc.add(value.sub(pivot_i.clone())?.exp()?)
        })?;

    pivot_i.add(sum.ln()?)
}

fn interval_sinh(x: SInterval) -> Fallible<SInterval> {
    SInterval::point(0.5)?.mul(x.clone().exp()?.sub(x.neg().exp()?)?)
}

fn check_renyi_order(alpha: f64) -> Fallible<()> {
    if !alpha.is_finite() || alpha <= 1.0 {
        return fallible!(
            FailedMap,
            "Renyi order alpha ({alpha}) must be finite and greater than one"
        );
    }
    Ok(())
}
