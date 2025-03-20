use std::collections::HashMap;

use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;

use crate::core::{Domain, Function, Measure, Measurement, Metric, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::Fallible;
use crate::metrics::{AbsoluteDistance, PartitionDistance};
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
    ) -> Fallible<Measurement<DI, DI::Carrier, MI, MO>>;
}

pub trait NoiseThresholdPrivacyMap<MI: Metric, MO: Measure>: Sample {
    fn noise_threshold_privacy_map(self, threshold: IBig) -> Fallible<PrivacyMap<MI, MO>>;
}

impl<MO: 'static + Measure, TK, const P: usize>
    MakeNoiseThreshold<
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        PartitionDistance<AbsoluteDistance<UBig>>,
        MO,
    > for ZExpFamily<P>
where
    TK: Hashable,
    ZExpFamily<P>: NoiseThresholdPrivacyMap<PartitionDistance<AbsoluteDistance<UBig>>, MO>,
{
    type Threshold = IBig;
    fn make_noise_threshold(
        self,
        (input_domain, input_metric): (
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            PartitionDistance<AbsoluteDistance<UBig>>,
        ),
        threshold: IBig,
    ) -> Fallible<
        Measurement<
            MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
            HashMap<TK, IBig>,
            PartitionDistance<AbsoluteDistance<UBig>>,
            MO,
        >,
    > {
        let distribution = self.clone();

        if self.scale < RBig::ZERO {
            return fallible!(FailedFunction, "scale must be non-negative");
        }

        Measurement::new(
            input_domain,
            Function::new_fallible(enclose!(
                (distribution, threshold),
                move |data: &HashMap<TK, IBig>| {
                    data.iter()
                        // noise output count
                        .map(|(k, v)| Ok((k.clone(), distribution.sample(v)?)))
                        // only keep keys with values gte threshold
                        .filter(|res| res.as_ref().map(|(_k, v)| v >= &threshold).unwrap_or(true))
                        // fail the whole computation if any noise addition failed
                        .collect()
                }
            )),
            input_metric,
            MO::default(),
            self.noise_threshold_privacy_map(threshold)?,
        )
    }
}
