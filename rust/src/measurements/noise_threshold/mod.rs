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
use crate::traits::Hashable;

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

pub trait NoiseThresholdPrivacyMap<MI: Metric, MO: Measure>: Sample {
    /// # Proof Definition
    /// Given a distribution `self`,
    /// returns `Err(e)` if `self` is not a valid distribution.
    /// Otherwise the output is `Ok(privacy_map)`
    /// where `privacy_map` observes the following:
    ///
    /// Define `function(x)` as a function that updates each pair `(k_i, v_i + Z_i)`
    /// where `Z_i` are iid samples from `self`,
    /// and discards pairs where `v_i + Z_i` has smaller magnitude than `|threshold|`.
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

        let inner = match threshold.sign() {
            Sign::Positive => Ordering::Less,
            Sign::Negative => Ordering::Greater,
        };

        Measurement::new(
            input_domain,
            input_metric,
            output_measure,
            Function::new_fallible(move |data: &HashMap<TK, IBig>| {
                data.into_iter()
                    // noise output count
                    .map(|(k, v)| Ok((k.clone(), self.sample(v)?)))
                    // only keep keys with values gte threshold, and errors
                    .filter(|r| r.as_ref().map_or(true, |p| p.1.cmp(&threshold) != inner))
                    // fail the whole computation if any noise addition failed
                    .collect()
            }),
            privacy_map,
        )
    }
}
