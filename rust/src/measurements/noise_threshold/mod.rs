use std::cmp::Ordering;
use std::collections::HashMap;

use dashu::base::Sign;
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use num::Zero;
use opendp_derive::proven;

use crate::core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, L0PInfDistance, majorizing_counts};
use crate::traits::{Hashable, InfCast, samplers::Shuffle};

use super::{Sample, ZExpFamily};

#[cfg(test)]
mod test;

mod distribution;
pub use distribution::*;

mod nature;

/// Make a measurement that samples from a random variable `RV`.
pub trait MakeNoiseThreshold<DI: Domain, MI: Metric, MO: Measure>
where
    (DI, MI): MetricSpace,
{
    type Threshold;

    /// # Proof Definition
    /// For any choice of arguments to `self`,
    /// returns a valid measurement or error.
    fn make_noise_threshold(
        self,
        input_space: (DI, MI),
        threshold: Self::Threshold,
    ) -> Fallible<Measurement<DI, MI, MO, DI::Carrier>>;
}

// Compute the probability of at least one of two independent events.
// Exact rational arithmetic avoids cancellation even for subnormal probabilities.
fn union_probability(a: f64, b: f64) -> Fallible<f64> {
    let a = RBig::try_from(a)?;
    let b = RBig::try_from(b)?;
    f64::inf_cast(&a + &b - a * b)
}

fn repeated_probability(mut p: f64, mut n: u32) -> Fallible<f64> {
    let mut out = 0.;
    while n != 0 {
        if n % 2 != 0 {
            out = union_probability(out, p)?;
        }
        n /= 2;
        if n != 0 {
            p = union_probability(p, p)?;
        }
    }
    Ok(out)
}

fn count(n: UBig) -> Fallible<u32> {
    u32::try_from(n).map_err(|_| err!(FailedMap, "partition count exceeds u32"))
}

/// Bound the probability of releasing any unstable key subject to integer
/// L0, L1 and L-infinity sensitivity bounds. The tail callback must upper
/// bound the tail of a symmetric log-concave discrete noise distribution.
/// With exact tails the bound is optimal before output postprocessing.
#[proven]
pub(crate) fn threshold_delta(
    l0: u32,
    l1: UBig,
    li: UBig,
    threshold: UBig,
    tail: impl Fn(UBig) -> Fallible<f64>,
) -> Fallible<f64> {
    let s = l1.min(&li * l0);
    let b = li.min(s.clone());
    if s.is_zero() {
        return Ok(0.);
    }
    if b > threshold {
        return fallible!(
            FailedMap,
            "threshold must not be smaller than l-infinity sensitivity {b}"
        );
    }
    let p = |a: &UBig| {
        let p = tail(&threshold - a)?;
        if !(0.0..=1.0).contains(&p) {
            return fallible!(FailedMap, "tail bound must be a probability");
        }
        Ok(p)
    };
    if b == UBig::ONE {
        return repeated_probability(p(&b)?, count(s)?);
    }

    let (q, r) = majorizing_counts(s.clone(), b.clone());
    let lo = count(q)? + u32::from(!r.is_zero());
    let hi = count(s.clone().min(UBig::from(l0)))?;
    let cap = &b - UBig::ONE;
    let remainder = |m: u32| (&s - UBig::from(m)) % &cap;

    // The objective is convex between consecutive breakpoints and affine
    // on the breakpoints (see threshold_delta.tex).
    let mut candidates = vec![lo, hi];
    let r_lo = remainder(lo);
    if r_lo <= UBig::from(hi - lo) {
        candidates.push(lo + count(r_lo)?);
    }
    let r_hi = remainder(hi);
    let back = if r_hi.is_zero() {
        UBig::ZERO
    } else {
        &cap - r_hi
    };
    if back <= UBig::from(hi - lo) {
        candidates.push(hi - count(back)?);
    }

    candidates.sort_unstable();
    candidates.dedup();
    let p_one = p(&UBig::ONE)?;
    let p_cap = p(&b)?;
    let mut delta: f64 = 0.;
    for m in candidates {
        let (q, r) = majorizing_counts(&s - UBig::from(m), cap.clone());
        let q = count(q)?;
        let has_remainder = u32::from(!r.is_zero());
        let mut d = repeated_probability(p_cap, q)?;
        if has_remainder != 0 {
            d = union_probability(d, p(&(r + UBig::ONE))?)?;
        }
        d = union_probability(d, repeated_probability(p_one, m - q - has_remainder)?)?;
        delta = delta.max(d);
    }
    Ok(delta)
}

pub trait NoiseThresholdPrivacyMap<MI: Metric, MO: Measure>: Sample {
    /// # Proof Definition
    /// Given a distribution `self`,
    /// returns `Err(e)` if `self` is not a valid distribution.
    /// Otherwise the output is `Ok(privacy_map)`
    /// where `privacy_map` observes the following:
    ///
    /// Define `function(x)` as a function that discards entries with value zero,
    /// then updates each remaining pair `(k_i, v_i + Z_i)` where `Z_i` are iid samples from `self`,
    /// and discards pairs whose noisy value does not strictly exceed the threshold in its signed direction.
    /// The ordering of returned pairs is independent from the input ordering.
    ///
    /// For every pair of elements $x, x'$ in `MapDomain<AtomDomain<TK>, AtomDomain<IBig>>`,
    /// and for every pair (`d_in`, `d_out`),
    /// where `d_in` has the associated type for `input_metric` and `d_out` has the associated type for `output_measure`,
    /// if $x, x'$ are `d_in`-close under `input_metric`, `privacy_map(d_in)` does not raise an exception,
    /// and `privacy_map(d_in) <= d_out`,
    /// then `function(x)`, `function(x')` are `d_out`-close under `output_measure`.
    fn noise_threshold_privacy_map(
        &self,
        input_metric: &MI,
        output_measure: &MO,
        threshold: UBig,
    ) -> Fallible<PrivacyMap<MI, MO>>;
}

#[proven(proof_path = "measurements/noise_threshold/MakeNoiseThreshold_IBig_for_RV.tex")]
impl<TK, const P: usize, const Q: usize, MO: 'static + Measure>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        L0PInfDistance<Q, AbsoluteDistance<RBig>>,
        MO,
    > for ZExpFamily<P>
where
    TK: Hashable,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<L0PInfDistance<Q, AbsoluteDistance<RBig>>, MO>,
{
    type Threshold = IBig;
    fn make_noise_threshold(
        self,
        (input_domain, input_metric): (
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            L0PInfDistance<Q, AbsoluteDistance<RBig>>,
        ),
        threshold: IBig,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            L0PInfDistance<Q, AbsoluteDistance<RBig>>,
            MO,
            HashMap<TK, IBig>,
        >,
    > {
        let output_measure = MO::default();
        let threshold_magnitude = threshold.clone().into_parts().1;
        let privacy_map =
            self.noise_threshold_privacy_map(&input_metric, &output_measure, threshold_magnitude)?;

        let keep_ordering = match threshold.sign() {
            Sign::Positive => Ordering::Greater,
            Sign::Negative => Ordering::Less,
        };

        Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::new_fallible(move |data: &HashMap<TK, IBig>| {
                let mut pairs: Vec<_> = data
                    .iter()
                    // Missing keys and explicit zero values are equivalent under the input metric.
                    .filter(|(_, v)| *v != &IBig::ZERO)
                    // noise output count
                    .map(|(k, v)| Ok((k.clone(), self.sample(v)?)))
                    // retain only values strictly beyond the threshold, and errors
                    .filter(|r| {
                        r.as_ref()
                            .map_or(true, |p| p.1.cmp(&threshold) == keep_ordering)
                    })
                    // fail the whole computation if any noise addition failed
                    .collect::<Fallible<_>>()?;

                // Hash collisions can preserve insertion order, even with a random hasher.
                pairs.shuffle()?;
                Ok(pairs.into_iter().collect())
            }),
            privacy_map,
        )
    }
}
