use dashu::{rational::RBig, rbig};
use num::Zero;
use opendp_derive::bootstrap;

use crate::{
    core::{Function, Measurement, PrivacyMap},
    domains::AtomDomain,
    error::Fallible,
    measures::{MultiDP, PrivacyGuarantee},
    metrics::AbsoluteDistance,
    traits::{
        InfCast, SInterval,
        backend::Dashu,
        samplers::{CanonicalRV, PartialSample},
    },
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

#[derive(Clone)]
struct CanonicalTradeoffPoint {
    one_minus_delta: RBig,
    exp_epsilon: RBig,
}

impl CanonicalTradeoffPoint {
    fn new(epsilon: f64, delta: f64) -> Fallible<Self> {
        // The software interval supplies an upward-rounded, exactly
        // representable rational upper bound for exp(epsilon). Keep its
        // reciprocal as an operation on the same rational: independently
        // rounding exp(-epsilon) would no longer describe an exactly
        // symmetric tradeoff function.
        let exp_epsilon = RBig::try_from(SInterval::<Dashu>::point(epsilon)?.exp()?.upper_f64()?)?;

        Ok(Self {
            one_minus_delta: RBig::ONE - RBig::try_from(delta)?,
            exp_epsilon,
        })
    }

    fn beta(&self, alpha: &RBig) -> RBig {
        let left = &self.one_minus_delta - &self.exp_epsilon * alpha;
        let right = (&self.one_minus_delta - alpha) / &self.exp_epsilon;
        left.max(right).max(RBig::ZERO)
    }

    fn fixed_point(&self) -> RBig {
        &self.one_minus_delta / (&self.exp_epsilon + RBig::ONE)
    }
}

#[derive(Clone)]
struct CanonicalTradeoff {
    points: Vec<CanonicalTradeoffPoint>,
}

impl CanonicalTradeoff {
    fn beta(&self, alpha: &RBig) -> RBig {
        self.points
            .iter()
            .map(|point| point.beta(alpha))
            .max()
            .unwrap_or(RBig::ZERO)
    }

    fn fixed_point(&self) -> RBig {
        self.points
            .iter()
            .map(CanonicalTradeoffPoint::fixed_point)
            .max()
            .unwrap_or(RBig::ZERO)
    }
}

// This grid controls approximation quality only. Every point is independently
// certified from the selected f-DP curve, so adding or removing grid points
// cannot weaken the privacy guarantee.
const CANONICAL_EPSILONS: &[f64] = &[
    0.0, 0.125, 0.25, 0.5, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 256.0, 512.0,
];

fn compile_tradeoff(d_out: &PrivacyGuarantee) -> Fallible<CanonicalTradeoff> {
    let points = CANONICAL_EPSILONS
        .iter()
        .map(|&epsilon| {
            let delta = d_out.symmetric_delta(epsilon)?;
            CanonicalTradeoffPoint::new(epsilon, delta)
        })
        .collect::<Fallible<Vec<_>>>()?;

    Ok(CanonicalTradeoff { points })
}

#[bootstrap(
    features("contrib", "honest-but-curious"),
    arguments(d_out(c_type = "AnyObject *", hint = "PrivacyGuarantee"))
)]
/// Make a Measurement that adds noise from the canonical noise distribution
/// associated with the supplied symmetric nontrivial f-DP tradeoff curve.
///
/// The supplied curve is sampled at a finite set of epsilon values. Each
/// sample is converted to a certified approximate-DP point, and the canonical
/// sampler uses the resulting exact rational polyhedral tradeoff curve. This
/// makes the fixed point used by the sampler exact; the epsilon grid affects
/// utility but not privacy safety.
///
/// # Citations
/// - [AV23 Canonical Noise Distributions and Private Hypothesis Tests](https://projecteuclid.org/journals/annals-of-statistics/volume-51/issue-2/Canonical-noise-distributions-and-private-hypothesis-tests/10.1214/23-AOS2259.short)
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric of the input.
/// * `d_in` - Sensitivity
/// * `d_out` - Privacy guarantee queried through its tradeoff-function view
///
/// # Why honest-but-curious?
/// The supplied tradeoff curve must be an explicitly symmetric, nontrivial
/// lower bound. This caller-verified precondition is required by the canonical
/// noise distribution theorem.
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

    let compiled_tradeoff = compile_tradeoff(&d_out)?;
    let fixed_point = compiled_tradeoff.fixed_point();

    // A fixed point at or above 1/2 corresponds to perfect privacy, while a
    // fixed point at zero is the trivial perfectly distinguishable curve. Both
    // are outside the nontrivial CND theorem and the former diverges in the
    // canonical quantile recursion.
    if fixed_point <= rbig!(0) || fixed_point >= rbig!(1 / 2) {
        return fallible!(
            MakeMeasurement,
            "fixed-point of the f-DP tradeoff curve must be strictly between 0 and 1/2"
        );
    }

    let r_d_in = RBig::try_from(d_in)?;
    let tradeoff_for_sampler = compiled_tradeoff.clone();
    let tradeoff_for_map = compiled_tradeoff.clone();

    Measurement::new(
        input_domain,
        input_metric,
        MultiDP::with_tradeoff(),
        Function::new_fallible(move |&arg: &f64| {
            let tradeoff_for_sample = tradeoff_for_sampler.clone();
            let tradeoff = move |alpha: RBig| Ok(tradeoff_for_sample.beta(&alpha));
            let canonical_rv = CanonicalRV {
                shift: RBig::try_from(arg.clamp(f64::MIN, f64::MAX)).unwrap_or(RBig::ZERO),
                scale: &r_d_in,
                tradeoff: &tradeoff,
                fixed_point: &fixed_point,
            };
            PartialSample::new(canonical_rv).value()
        }),
        PrivacyMap::new_fallible(move |d_in_p: &f64| {
            if !(0.0..=d_in).contains(d_in_p) {
                return fallible!(
                    FailedMap,
                    "d_in from the map ({d_in_p}) must be in [0, {d_in}]"
                );
            }
            if d_in_p.is_zero() {
                return PrivacyGuarantee::new().with_symmetric_tradeoff(|alpha| Ok(1.0 - alpha));
            }

            let tradeoff = tradeoff_for_map.clone();
            PrivacyGuarantee::new().with_symmetric_tradeoff(move |alpha| {
                let alpha = RBig::try_from(alpha)?;
                f64::neg_inf_cast(tradeoff.beta(&alpha))
            })
        }),
    )
}
