use core::f64;

use dashu::{base::Signed, rational::RBig};
use opendp_derive::{bootstrap, proven};

use crate::core::{Measure, Metric, MetricSpace};
use crate::measurements::noise::nature::Nature;
use crate::measurements::{MakeNoise, NoiseDomain, NoisePrivacyMap, ZExpFamily};
use crate::measures::{PrivacyCurve, PrivacyCurveDP};
use crate::{
    core::{Domain, Measurement, PrivacyMap},
    error::Fallible,
    measures::MaxDivergence,
    metrics::L1Distance,
    traits::{DInterval, InfCast},
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
/// * `MO` - Measure used to quantify privacy loss. Valid values are just `MaxDivergence`
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
    DiscreteLaplace { scale, k }.make_noise((input_domain, input_metric), MO::default())
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
    fn make_noise(
        self,
        input_space: (DI, MI),
        output_measure: MO,
    ) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>> {
        DI::Atom::new_distribution(self.scale, self.k)?.make_noise(input_space, output_measure)
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

// #[proven(
//     proof_path = "measurements/noise/distribution/laplace/NoisePrivacyMap_for_ZExpFamily1_PrivacyCurveDP.tex"
// )]
impl NoisePrivacyMap<L1Distance<RBig>, PrivacyCurveDP> for ZExpFamily<1> {
    fn noise_privacy_map(
        &self,
        _input_metric: &L1Distance<RBig>,
        _output_measure: &PrivacyCurveDP,
    ) -> Fallible<PrivacyMap<L1Distance<RBig>, PrivacyCurveDP>> {
        let ZExpFamily { scale } = self.clone();

        if scale < RBig::ZERO {
            return fallible!(MakeMeasurement, "scale ({scale}) must not be negative");
        }

        Ok(PrivacyMap::new_fallible(move |d_in: &RBig| {
            if d_in.is_negative() {
                return fallible!(FailedMap, "sensitivity ({d_in}) must be non-negative");
            }

            if d_in.is_zero() {
                return PrivacyCurve::new()
                    .with_approxDP(vec![(0.0, 0.0)])?
                    .with_zCDP(0.0)?
                    .with_renyiDP_trusted(move |_alpha| Ok(0.0));
            }

            if scale.is_zero() {
                return fallible!(
                    FailedMap,
                    "nonzero sensitivity with zero scale has no finite privacy guarantee"
                );
            }

            let epsilon = f64::inf_cast(d_in / scale.clone())?;
            let sensitivity = f64::inf_cast(d_in.clone())?;
            let scale_f = f64::inf_cast(scale.clone())?;

            if !epsilon.is_finite() {
                return fallible!(
                    FailedMap,
                    "epsilon ({epsilon}) must be finite for PrivacyCurveDP"
                );
            }
            if !sensitivity.is_finite() {
                return fallible!(
                    FailedMap,
                    "sensitivity ({sensitivity}) must be finite for PrivacyCurveDP"
                );
            }
            if !scale_f.is_finite() || scale_f <= 0.0 {
                return fallible!(
                    FailedMap,
                    "scale ({scale_f}) must be finite and positive for PrivacyCurveDP"
                );
            }

            let rho = zcdp_discrete_laplace(epsilon, sensitivity, scale_f)?;

            PrivacyCurve::new()
                // Trivial pure-DP certificate.
                .with_approxDP(vec![(epsilon, 0.0)])?
                // Tight zCDP certificate from Harrison-Manurangsi.
                .with_zCDP(rho)?
                // Exact discrete-Laplace RDP curve, with pure-DP RDP fallback if
                // numerical evaluation fails.
                .with_renyiDP_trusted(move |alpha| {
                    rdp_discrete_laplace(alpha, sensitivity, scale_f)
                        .or_else(|_| rdp_from_pureDP(alpha, epsilon))
                })
        }))
    }
}

fn zcdp_discrete_laplace(epsilon: f64, sensitivity: f64, scale: f64) -> Fallible<f64> {
    if epsilon == 0.0 || sensitivity == 0.0 {
        return Ok(0.0);
    }

    let a = 1.0 / scale;
    if !a.is_finite() || a > 700.0 {
        return Ok(epsilon.next_up());
    }

    let epsilon_i = DInterval::point(epsilon)?;
    let sensitivity_i = DInterval::from_approx(sensitivity)?;
    let a_i = DInterval::from_approx(a)?;

    let one_minus_exp_neg_eps =
        DInterval::point(0.0)?.sub(DInterval::point(-epsilon)?.exp_m1()?)?;
    let sinh_a = interval_sinh(a_i)?;
    let correction = one_minus_exp_neg_eps.div(sensitivity_i.mul(sinh_a)?)?;

    let rho = epsilon_i
        .mul(DInterval::point(1.0)?.sub(correction)?)?
        .clamp01_nonnegative_upper(epsilon)?;

    if rho.is_nan() || rho < 0.0 {
        return fallible!(
            FailedMap,
            "computed zCDP rho ({rho}) must be non-negative and not NaN"
        );
    }

    Ok(rho)
}

fn rdp_discrete_laplace(alpha: f64, sensitivity: f64, scale: f64) -> Fallible<f64> {
    check_renyi_order(alpha)?;

    if sensitivity == 0.0 {
        return Ok(0.0);
    }

    let a = 1.0 / scale;
    if !a.is_finite() || a > 700.0 {
        return fallible!(FailedMap, "exact discrete-Laplace RDP numerics overflowed");
    }
    if a == 0.0 {
        return Ok(0.0);
    }

    let alpha_i = DInterval::point(alpha)?;
    let a_i = DInterval::from_approx(a)?;
    let d_i = DInterval::from_approx(sensitivity)?;
    let one = DInterval::point(1.0)?;
    let two = DInterval::point(2.0)?;

    let exp_a = a_i.clone().exp()?;
    let log_expm1_a = a_i.clone().exp_m1()?.ln()?;
    let log_tanh = log_expm1_a
        .clone()
        .sub(exp_a.clone().add(one.clone())?.ln()?)?;

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
        .neg()?
        .sub(log_expm1_a.clone())?;

    // Original term:
    // (exp(a - a alpha d) - exp(a(alpha(d + 2) - d)))
    // / (exp(a) - exp(2 a alpha))
    //
    // Both numerator and denominator are negative, so rewrite as:
    // (exp(B) - exp(A)) / (exp(2a alpha) - exp(a)).
    let num_hi = a_i.clone().mul(
        alpha_i
            .clone()
            .mul(d_i.clone().add(two.clone())?)?
            .sub(d_i.clone())?,
    )?;
    let num_lo = a_i
        .clone()
        .sub(a_i.clone().mul(alpha_i.clone())?.mul(d_i.clone())?)?;
    let den_hi = two.mul(a_i.clone())?.mul(alpha_i.clone())?;
    let den_lo = a_i.clone();

    let log_t2 = log_sub_exp(num_hi, num_lo)?.sub(log_sub_exp(den_hi, den_lo)?)?;

    let log_t3 = a_i
        .mul(alpha_i.clone().sub(one.clone())?)?
        .mul(d_i)?
        .sub(log_expm1_a)?;

    let log_sum = log_sum_exp3(log_t1, log_t2, log_t3)?;
    let log_moment = log_tanh.add(log_sum)?;

    let epsilon = log_moment.div(alpha_i.sub(one)?)?.upper_f64()?;

    if epsilon.is_nan() || epsilon < 0.0 {
        return fallible!(
            FailedMap,
            "computed RDP epsilon ({epsilon}) must be non-negative and not NaN"
        );
    }

    Ok(epsilon)
}

fn rdp_from_pureDP(alpha: f64, epsilon: f64) -> Fallible<f64> {
    check_renyi_order(alpha)?;

    if epsilon == 0.0 {
        return Ok(0.0);
    }

    let alpha_i = DInterval::point(alpha)?;
    let epsilon_i = DInterval::from_approx(epsilon)?;
    let one = DInterval::point(1.0)?;

    let log_num = log_sum_exp2(
        alpha_i.clone().mul(epsilon_i.clone())?,
        one.clone().sub(alpha_i.clone())?.mul(epsilon_i.clone())?,
    )?;
    let log_den = log_sum_exp2(epsilon_i, DInterval::point(0.0)?)?;

    let rdp = log_num.sub(log_den)?.div(alpha_i.sub(one)?)?.upper_f64()?;

    if rdp.is_nan() || rdp < 0.0 {
        return fallible!(
            FailedMap,
            "computed pure-DP RDP epsilon ({rdp}) must be non-negative and not NaN"
        );
    }

    Ok(rdp)
}

fn log_sum_exp2(a: DInterval, b: DInterval) -> Fallible<DInterval> {
    log_sum_exp_many([a, b])
}

fn log_sum_exp3(a: DInterval, b: DInterval, c: DInterval) -> Fallible<DInterval> {
    log_sum_exp_many([a, b, c])
}

fn log_sum_exp_many<const N: usize>(values: [DInterval; N]) -> Fallible<DInterval> {
    let mut pivot = f64::NEG_INFINITY;
    for value in &values {
        pivot = pivot.max(value.upper_f64()?);
    }
    let pivot_i = DInterval::point(pivot)?;

    let sum = values
        .into_iter()
        .try_fold(DInterval::point(0.0)?, |acc, value| {
            acc.add(value.sub(pivot_i.clone())?.exp()?)
        })?;

    pivot_i.add(sum.ln()?)
}

fn log_sub_exp(hi: DInterval, lo: DInterval) -> Fallible<DInterval> {
    let z = lo.sub(hi.clone())?;

    if z.upper_f64()? >= 0.0 {
        return Ok(hi);
    }

    let one_minus_exp_z = DInterval::point(0.0)?.sub(z.exp_m1()?)?;
    hi.add(one_minus_exp_z.ln()?)
}

fn interval_sinh(x: DInterval) -> Fallible<DInterval> {
    DInterval::point(0.5)?.mul(x.clone().exp()?.sub(x.neg()?.exp()?)?)
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

trait ClampNonnegativeUpper {
    fn clamp01_nonnegative_upper(self, upper_cap: f64) -> Fallible<f64>;
}

impl ClampNonnegativeUpper for DInterval {
    fn clamp01_nonnegative_upper(self, upper_cap: f64) -> Fallible<f64> {
        self.upper_f64().map(|value| value.clamp(0.0, upper_cap))
    }
}
