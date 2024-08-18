use std::collections::HashMap;

use dashu::integer::IBig;
use dashu::rational::RBig;
use num::Zero;

use crate::core::{Function, Measurement, MetricSpace, PrivacyMap};
use crate::domains::{AtomDomain, MapDomain};
use crate::error::{Error, ErrorVariant, Fallible};
use crate::measures::{Approximate, MaxDivergence};
use crate::metrics::L1Distance;
use crate::traits::samplers::sample_discrete_laplace;
use crate::traits::{Hashable, InfCast, InfDiv, InfExp, InfLn, InfSub};

#[cfg(all(test, feature = "partials"))]
mod test;


/// Make a Measurement that uses propose-test-release to privatize a hashmap of counts.
///
/// This function takes a noise granularity in terms of 2^k.
/// Larger granularities are more computationally efficient, but have a looser privacy map.
/// If k is not set, k defaults to the smallest granularity.
///
/// # Arguments
/// * `input_domain` - Domain of the input.
/// * `input_metric` - Metric for the input domain.
/// * `scale` - Noise scale parameter for the laplace distribution
/// * `threshold` - Exclude counts that are less than this minimum value.
///
/// # Generics
/// * `TK` - Type of Key. Must be hashable/categorical.
pub fn make_bigint_laplace_threshold<TK>(
    (input_domain, input_metric): (MapDomain<AtomDomain<TK>, AtomDomain<IBig>>, L1Distance<RBig>),
    scale: RBig,
    threshold: IBig,
) -> Fallible<
    Measurement<
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        HashMap<TK, IBig>,
        L1Distance<RBig>,
        Approximate<MaxDivergence>,
    >,
>
where
    TK: Hashable,
    (
        MapDomain<AtomDomain<TK>, AtomDomain<IBig>>,
        L1Distance<RBig>,
    ): MetricSpace,
{
    if scale < RBig::ZERO {
        return fallible!(FailedFunction, "scale must be non-negative");
    }
    let f_scale = f64::neg_inf_cast(scale.clone())?;
    let f_threshold = f64::neg_inf_cast(threshold.clone())?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |data: &HashMap<TK, IBig>| {
            data.clone()
                .into_iter()
                // noise output count
                .map(|(k, v)| sample_discrete_laplace(scale.clone()).map(|s| (k, v + s)))
                // only keep keys with values gte threshold
                .filter(|res| res.as_ref().map(|(_k, v)| v >= &threshold).unwrap_or(true))
                // fail the whole computation if any cast or noise addition failed
                .collect()
        }),
        input_metric,
        Approximate::default(),
        PrivacyMap::new_fallible(move |d_in: &RBig| {
            let d_in = f64::inf_cast(d_in.clone())?;
            if d_in.is_sign_negative() {
                return fallible!(FailedMap, "d_in must be not be negative");
            }

            if d_in.is_zero() {
                return Ok((0.0, 0.0));
            }

            if f_scale.is_zero() {
                return Ok((f64::INFINITY, 1.0));
            }

            let epsilon = d_in.inf_div(&f_scale)?;

            // delta is the probability that noise will push the count beyond the threshold
            // δ = d_in / exp(distance_to_instability) / 2

            // however, computing exp is unstable, so it is computed last
            // δ = exp(ln(d_in / 2) - distance_to_instability)

            // compute the distance to instability, conservatively rounding down
            //                         = (threshold -            d_in)             / scale
            let distance_to_instability = f_threshold.neg_inf_sub(&d_in)?.neg_inf_div(&f_scale)?;

            if distance_to_instability <= 1.0 {
                return Ok((epsilon, 1.0));
            }

            // ln(delta) = ln(d_in / 2) - distance_to_instability
            let ln_delta = d_in
                .inf_div(&2.0)?
                .inf_ln()?
                .inf_sub(&distance_to_instability)?;

            // delta =        exp(ln(delta))
            let delta = match ln_delta.inf_exp() {
                // catch error on overflowing delta as just infinity
                Err(Error {
                    variant: ErrorVariant::Overflow,
                    ..
                }) => f64::INFINITY,
                result => result?,
            };

            // delta is only sensibly at most 1
            Ok((epsilon, delta.min(1.0)))
        }),
    )
}
