use std::cmp::Ordering;
use std::collections::HashMap;

use dashu::base::Sign;
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use opendp_derive::proven;

use crate::core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, L0PInfDistance};
use crate::traits::{Hashable, samplers::Shuffle};

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

/// Retain only values that satisfy the threshold in the returned native type.
/// This deterministic filter is privacy-preserving postprocessing.
pub(crate) fn then_filter_strict_threshold<K: Hashable, V: Clone + PartialOrd + 'static>(
    threshold: V,
    negative: bool,
) -> Function<HashMap<K, V>, HashMap<K, V>> {
    Function::new(move |data: &HashMap<K, V>| {
        let desired = if negative { Ordering::Less } else { Ordering::Greater };
        data.iter()
            .filter(|(_, value)| (*value).partial_cmp(&threshold) == Some(desired))
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect()
    })
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
impl<TK, const P: usize, MO: 'static + Measure>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        L0PInfDistance<P, AbsoluteDistance<RBig>>,
        MO,
    > for ZExpFamily<P>
where
    TK: Hashable,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<L0PInfDistance<P, AbsoluteDistance<RBig>>, MO>,
{
    type Threshold = IBig;
    fn make_noise_threshold(
        self,
        (input_domain, input_metric): (
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            L0PInfDistance<P, AbsoluteDistance<RBig>>,
        ),
        threshold: IBig,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            L0PInfDistance<P, AbsoluteDistance<RBig>>,
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
