use std::sync::Arc;

use dashu::{rational::RBig, rbig};
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{MultiDP, PrivacyGuarantee, PrivacyProfile},
    metrics::AbsoluteDistance,
    traits::{SInterval, samplers::PartialSample},
};

#[cfg(feature = "ffi")]
mod ffi;

mod sampling;
use sampling::CanonicalRV;

#[cfg(test)]
mod test;

// A rational lower bound on exp(epsilon) - 1.
fn exp_m1_lower(epsilon: f64) -> Fallible<RBig> {
    if epsilon >= 1024.0 {
        return Ok(RBig::try_from(f64::MAX)?);
    }
    let a = SInterval::point(epsilon)?.exp_m1()?.lower_f64()?;
    Ok(RBig::try_from(a.max(epsilon))?)
}

// On [0, c], D(u) = 1 - f(u) - u is the lower envelope of a*u + delta.
struct Segment {
    start: RBig,
    a: RBig,
    delta: RBig,
}

impl Segment {
    fn eval(&self, u: &RBig) -> RBig {
        &self.a * u + &self.delta
    }
}

struct CanonicalTradeoff {
    segments: Vec<Segment>,
    fixed_point: RBig,
}

impl CanonicalTradeoff {
    fn new(samples: &[(f64, f64)]) -> Fallible<Self> {
        let mut lines = samples
            .iter()
            .map(|&(epsilon, delta)| {
                if !epsilon.is_finite() || epsilon < 0.0 || !(0.0..=1.0).contains(&delta) {
                    return fallible!(MakeMeasurement, "invalid canonical noise constraint");
                }
                Ok((exp_m1_lower(epsilon)?, RBig::try_from(delta)?))
            })
            .collect::<Fallible<Vec<_>>>()?;
        let c = lines
            .iter()
            .map(|(a, delta)| (RBig::ONE - delta) / (rbig!(2) + a))
            .max()
            .unwrap_or(RBig::ZERO);
        if c <= RBig::ZERO || c >= rbig!(1 / 2) {
            return fallible!(
                MakeMeasurement,
                "canonical tradeoff fixed point must be strictly between 0 and 1/2"
            );
        }

        // Decreasing slopes, keeping only the smallest intercept for each.
        lines.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
        lines.dedup_by(|a, b| a.0 == b.0);
        let mut segments: Vec<Segment> = Vec::new();
        for (a, delta) in lines {
            let start = loop {
                let Some(last) = segments.last() else {
                    break RBig::ZERO;
                };
                let x = (&delta - &last.delta) / (&last.a - &a);
                if x > last.start {
                    break x;
                }
                segments.pop();
            };
            if start < c {
                segments.push(Segment { start, a, delta });
            }
        }
        Ok(Self {
            segments,
            fixed_point: c,
        })
    }

    fn profile(self: &Arc<Self>) -> Fallible<PrivacyProfile> {
        let tradeoff = self.clone();
        // The intercept at zero is an original, exactly representable delta.
        let slack = self.segments[0].delta.to_f64().value();
        PrivacyProfile::new_log(move |epsilons| {
            epsilons
                .iter()
                .map(|&epsilon| {
                    let a = exp_m1_lower(epsilon)?;
                    // D(u) - a*u is concave. Its maximum occurs where the
                    // slope first falls to a or below, or at the fixed point.
                    let index = tradeoff.segments.partition_point(|s| s.a > a);
                    let core = match tradeoff.segments.get(index) {
                        Some(s) => s.eval(&s.start) - &a * &s.start,
                        None => RBig::ONE - (rbig!(2) + &a) * &tradeoff.fixed_point,
                    } - &tradeoff.segments[0].delta;
                    if core.is_zero() {
                        return Ok(f64::NEG_INFINITY);
                    }
                    Ok(SInterval::from_rational(&core)?.ln()?.upper_f64()?.min(0.0))
                })
                .collect()
        })
        .with_delta_slack(slack)
    }
}

const CANONICAL_EPSILONS: &[f64] = &[
    0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0,
];

fn compile_profile(specification: &PrivacyGuarantee) -> Fallible<CanonicalTradeoff> {
    let mut epsilons = CANONICAL_EPSILONS.to_vec();
    if let Some((epsilon, _)) = specification.approx_dp() {
        epsilons.push(epsilon);
    }
    epsilons.sort_by(f64::total_cmp);
    epsilons.dedup();
    let deltas = specification.delta_batch(&epsilons)?;
    let tv = deltas[0];
    let mut samples = epsilons.into_iter().zip(deltas).collect::<Vec<_>>();
    if tv > 0.0 && tv < 0.125 {
        let extra = [0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0]
            .map(|factor| tv * factor)
            .into_iter()
            .filter(|&epsilon| epsilon > 0.0 && epsilon < 0.125)
            .collect::<Vec<_>>();
        let deltas = specification.delta_batch(&extra)?;
        samples.extend(extra.into_iter().zip(deltas));
    }
    CanonicalTradeoff::new(&samples)
}

#[bootstrap(
    features("contrib"),
    arguments(d_out(c_type = "AnyObject *", hint = "PrivacyGuarantee"))
)]
/// Make a Measurement that adds canonical noise compiled from a privacy guarantee.
///
/// Conservative delta bounds are sampled on a finite ordered epsilon grid,
/// including an exact approximate-DP certificate's endpoint when available.
/// For small privacy loss, the grid is supplemented near the total variation bound.
/// Each sample defines an exact rational symmetric tradeoff constraint. Only
/// the lower-half drift and its fixed point are compiled. Degenerate
/// perfect-privacy and trivial curves are rejected.
///
/// The output guarantee contains only the log-native profile implied by the
/// compiled tradeoff envelope. It does not retain the specification's
/// RDP, zCDP or Gaussian-DP representations. Only the sampled constraints
/// are enforced; the resulting guarantee can be weaker than the specification
/// at unsampled epsilon values. Use the returned privacy map for accounting.
/// Tail traversal uses certified affine iteration with bounded-size rational
/// arithmetic; refining the uniform sample also refines the quantile bounds.
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
    let compiled = Arc::new(compile_profile(&d_out)?);
    let profile = compiled.profile()?;
    let scale = RBig::try_from(d_in)?;
    Measurement::new(
        input_domain,
        input_metric,
        MultiDP::default(),
        Function::new_fallible(move |&arg: &f64| {
            let arg = arg.clamp(f64::MIN, f64::MAX);
            if scale.is_zero() {
                return Ok(arg);
            }
            let canonical_rv = CanonicalRV {
                shift: RBig::try_from(arg)?,
                scale: &scale,
                tradeoff: &compiled,
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
