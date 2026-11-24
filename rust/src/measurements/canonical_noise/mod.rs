use dashu::{rational::RBig, rbig};
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{MultiDP, PrivacyGuarantee, PrivacyProfile, Purity},
    metrics::AbsoluteDistance,
    traits::{
        SInterval,
        samplers::{CanonicalRV, PartialSample},
    },
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// One exact symmetric approximate-DP constraint, retaining the sampled values.
#[derive(Clone)]
struct ApproxDPConstraint {
    epsilon: f64,
    delta: f64,
    one_minus_delta: RBig,
    q: RBig,
}

impl ApproxDPConstraint {
    fn new(epsilon: f64, delta: f64) -> Fallible<Self> {
        if !epsilon.is_finite() || epsilon < 0.0 || !(0.0..=1.0).contains(&delta) {
            return fallible!(
                MakeMeasurement,
                "canonical noise requires finite nonnegative epsilon and delta in [0, 1]"
            );
        }
        // Dashu's binary exponent is an isize. Do not attempt an exponential
        // whose exponent is outside that working range.
        if epsilon >= (isize::MAX as f64) * std::f64::consts::LN_2 {
            return fallible!(
                NumericError,
                "canonical noise epsilon exceeds the supported exponential range"
            );
        }
        // A LOWER enclosure makes the compiled privacy constraint stronger.
        // Use the exact reciprocal of this same rational, never exp(-epsilon).
        let q = SInterval::point(epsilon)?.exp()?.lower_f64()?;
        if !q.is_finite() || q < 1.0 {
            return fallible!(
                NumericError,
                "canonical noise exponential bound must be finite and at least one"
            );
        }
        Ok(Self {
            epsilon,
            delta,
            one_minus_delta: RBig::ONE - RBig::try_from(delta)?,
            q: RBig::try_from(q)?,
        })
    }

    fn beta(&self, alpha: &RBig) -> RBig {
        let left = &self.one_minus_delta - &self.q * alpha;
        let right = (&self.one_minus_delta - alpha) / &self.q;
        left.max(right).max(RBig::ZERO)
    }

    fn fixed_point(&self) -> RBig {
        &self.one_minus_delta / (&self.q + RBig::ONE)
    }
}

/// Private exact envelope used only by canonical-noise compilation/sampling.
#[derive(Clone)]
struct PolyhedralTradeoff {
    constraints: Vec<ApproxDPConstraint>,
    fixed_point: RBig,
}

impl PolyhedralTradeoff {
    fn new(samples: &[(f64, f64)]) -> Fallible<Self> {
        if samples.is_empty() || samples.windows(2).any(|pair| pair[0].0 > pair[1].0) {
            return fallible!(
                MakeMeasurement,
                "canonical noise samples must be nonempty and ordered by epsilon"
            );
        }
        let constraints = samples
            .iter()
            .map(|&(epsilon, delta)| ApproxDPConstraint::new(epsilon, delta))
            .collect::<Fallible<Vec<_>>>()?;
        // For nonincreasing components, the maximum of their fixed points is
        // the fixed point of their maximum. All operations here are rational.
        let fixed_point = constraints
            .iter()
            .map(ApproxDPConstraint::fixed_point)
            .max()
            .unwrap();
        if fixed_point <= RBig::ZERO || fixed_point >= rbig!(1 / 2) {
            return fallible!(
                MakeMeasurement,
                "compiled canonical tradeoff fixed point must be strictly between 0 and 1/2"
            );
        }
        Ok(Self {
            constraints,
            fixed_point,
        })
    }

    fn beta(&self, alpha: &RBig) -> RBig {
        self.constraints
            .iter()
            .map(|point| point.beta(alpha))
            .max()
            .unwrap()
    }

    fn profile(&self) -> Fallible<PrivacyProfile> {
        // Retain the exact final minimum as existing profile-owned slack.
        // The log core represents S(epsilon) - slack. This avoids perturbing
        // the final sampled delta when converting to/from logarithms.
        let slack = self
            .constraints
            .iter()
            .map(|point| point.delta)
            .reduce(f64::min)
            .unwrap();
        let log_core = |delta: f64| -> Fallible<f64> {
            if delta == slack {
                return Ok(f64::NEG_INFINITY);
            }
            (SInterval::point(delta)? - SInterval::point(slack)?)?
                .ln()?
                .upper_f64()
        };
        let initial = log_core(1.0)?;
        let mut minimum = 1.0_f64;
        let mut steps = Vec::with_capacity(self.constraints.len());
        for point in &self.constraints {
            minimum = minimum.min(point.delta);
            steps.push((point.epsilon, log_core(minimum)?));
        }
        PrivacyProfile::new_log(move |epsilons| {
            let mut index = 0;
            let mut value = initial;
            Ok(epsilons
                .iter()
                .map(|&epsilon| {
                    while index < steps.len() && steps[index].0 <= epsilon {
                        value = steps[index].1;
                        index += 1;
                    }
                    value
                })
                .collect())
        })
        .with_delta_slack(slack)
    }
}

// This finite grid controls utility, not certification. Every sampled bound is
// retained exactly; accessible certificate endpoints are added below.
const CANONICAL_EPSILONS: &[f64] = &[
    0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0,
];

fn compile_profile(specification: &PrivacyGuarantee) -> Fallible<PolyhedralTradeoff> {
    let mut epsilons = CANONICAL_EPSILONS.to_vec();
    if let Some((epsilon, _)) = specification.approx_dp() {
        epsilons.push(epsilon);
    }
    epsilons.sort_by(f64::total_cmp);
    epsilons.dedup();
    let samples = epsilons
        .into_iter()
        .map(|epsilon| Ok((epsilon, specification.delta(epsilon)?)))
        .collect::<Fallible<Vec<_>>>()?;
    PolyhedralTradeoff::new(&samples)
}

#[bootstrap(
    features("contrib"),
    arguments(d_out(c_type = "AnyObject *", hint = "PrivacyGuarantee"))
)]
/// Make a Measurement that adds canonical noise compiled from a privacy guarantee.
///
/// Conservative delta bounds are sampled on a finite ordered epsilon grid,
/// including an exact approximate-DP certificate's endpoint when available.
/// Each sample defines an exact rational symmetric tradeoff constraint; their
/// maximum and its fixed point are compiled once at construction. Degenerate
/// perfect-privacy and trivial curves are rejected.
///
/// The output guarantee contains only a log-native step profile, formed from
/// prefix minima of the sampled deltas. It does not retain the specification's
/// RDP, zCDP or Gaussian-DP representations. The grid affects utility, not safety.
/// Exact tail traversal can be expensive near perfect privacy, especially for
/// zero-epsilon specifications with very small positive delta.
///
/// # Citations
/// - [AV23 Canonical Noise Distributions and Private Hypothesis Tests](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Canonical-noise-distributions-and-private-hypothesis-tests/10.1214/23-AOS2259.short)
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `d_in` - Finite nonnegative sensitivity.
/// * `d_out` - Desired privacy guarantee, queried for conservative delta bounds.
pub fn make_canonical_noise(
    input_domain: AtomDomain<f64>,
    input_metric: AbsoluteDistance<f64>,
    d_in: f64,
    d_out: PrivacyGuarantee,
) -> Fallible<Measurement<AtomDomain<f64>, AbsoluteDistance<f64>, MultiDP, f64>> {
    if input_domain.nan() {
        return fallible!(MakeMeasurement, "input_domain must consist of non-nan data");
    }
    if d_in.is_sign_negative() || !d_in.is_finite() {
        return fallible!(
            MakeMeasurement,
            "d_in ({d_in}) must be a finite non-negative number"
        );
    }
    let compiled = compile_profile(&d_out)?;
    let profile = compiled.profile()?;
    let purity = if d_in == 0.0 || compiled.constraints.iter().any(|point| point.delta == 0.0) {
        Purity::Pure
    } else {
        Purity::Approximate
    };
    let scale = RBig::try_from(d_in)?;
    Measurement::new(
        input_domain,
        input_metric,
        MultiDP::with_profile(purity),
        Function::new_fallible(move |&arg: &f64| {
            let tradeoff = |alpha: RBig| Ok(compiled.beta(&alpha));
            let canonical_rv = CanonicalRV {
                shift: RBig::try_from(arg.clamp(f64::MIN, f64::MAX))?,
                scale: &scale,
                tradeoff: &tradeoff,
                fixed_point: &compiled.fixed_point,
            };
            PartialSample::new(canonical_rv).value()
        }),
        PrivacyMap::new_fallible(move |distance: &f64| {
            if !(0.0..=d_in).contains(distance) {
                return fallible!(
                    FailedMap,
                    "d_in from the map ({distance}) must be in [0, {d_in}]"
                );
            }
            let profile = if *distance == 0.0 {
                PrivacyProfile::new_log(|epsilons| Ok(vec![f64::NEG_INFINITY; epsilons.len()]))
            } else {
                profile.clone()
            };
            Ok(PrivacyGuarantee::new().with_profile(profile))
        }),
    )
}
