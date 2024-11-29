use crate::{
    core::{Function, Measure, Measurement, PrivacyMap},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    measures::{MaxDivergence, RangeDivergence},
    metrics::LInfDistance,
    traits::{
        InfCast, InfDiv, Number,
        samplers::{ExponentialRV, GumbelRV, InverseCDF, PartialSample},
    },
};
use dashu::float::FBig;
use num::Zero;
use opendp_derive::bootstrap;
use std::fmt::Display;

#[cfg(feature = "ffi")]
mod ffi;
#[cfg(test)]
mod test;

#[bootstrap(
    features("contrib"),
    arguments(
        optimize(c_type = "char *", rust_type = "String"),
        output_measure(c_type = "AnyMeasure *", rust_type = b"null"),
    ),
    generics(MO(suppress), TIA(suppress))
)]
/// Make a Measurement that takes a vector of scores and privately selects the index of the highest score.
///
/// # Arguments
/// * `input_domain` - Domain of the input vector. Must be a non-nullable `VectorDomain`
/// * `input_metric` - Metric on the input domain. Must be `LInfDistance`
/// * `output_measure` - One of `MaxDivergence` or `RangeDivergence`
/// * `scale` - Scale for the noise distribution
/// * `optimize` - Indicate whether to privately return the "max" or "min"
///
/// # Generics
/// * `MO` - Output measure. Must be one of `MaxDivergence` or `RangeDivergence`
/// * `TIA` - Atom Input Type. Type of each element in the score vector
pub fn make_report_noisy_max<MO: SelectionMeasure, TIA>(
    input_domain: VectorDomain<AtomDomain<TIA>>,
    input_metric: LInfDistance<TIA>,
    output_measure: MO,
    scale: f64,
    optimize: Optimize,
) -> Fallible<Measurement<VectorDomain<AtomDomain<TIA>>, usize, LInfDistance<TIA>, MO>>
where
    TIA: Number,
    FBig: TryFrom<TIA> + TryFrom<f64>,
    f64: InfCast<TIA>,
{
    if input_domain.element_domain.nan() {
        return fallible!(
            MakeMeasurement,
            "input_domain member elements must not be nan"
        );
    }

    if scale.is_sign_negative() {
        return fallible!(MakeMeasurement, "scale ({}) must not be negative", scale);
    }

    let f_scale = FBig::try_from(scale.clone())
        .map_err(|_| err!(MakeMeasurement, "scale ({}) must be finite", scale))?;

    Measurement::new(
        input_domain,
        Function::new_fallible(move |x: &Vec<TIA>| {
            if scale.is_zero() {
                let cmp = |l: &TIA, r: &TIA| match optimize {
                    Optimize::Max => l > r,
                    Optimize::Min => l < r,
                };
                return Ok((x.iter().enumerate())
                    .reduce(|l, r| if cmp(&l.1, &r.1) { l } else { r })
                    .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                    .0);
            }

            (x.iter().enumerate())
                // Cast to FBig and discard failed casts.
                // Cast only fails on NaN scores, which are not in the input domain but could still be passed by the user.
                // If the user still passes NaN in the input data, discarding results in graceful failure.
                .filter_map(|(i, x_i)| Some((i, FBig::try_from(*x_i).ok()?)))
                // Normalize sign.
                .map(|(i, x_i)| {
                    let y_i = match optimize {
                        Optimize::Min => -x_i,
                        Optimize::Max => x_i,
                    };
                    (i, y_i)
                })
                // Initialize partial sample.
                .map(|(i, f_shift)| {
                    let rv = MO::random_variable(f_shift, f_scale.clone())?;
                    Ok((i, PartialSample::new(rv)))
                })
                // Reduce to the pair with largest sample.
                .reduce(|l, r| {
                    let (mut l, mut r) = (l?, r?);
                    Ok(if l.1.greater_than(&mut r.1)? { l } else { r })
                })
                .ok_or_else(|| err!(FailedFunction, "there must be at least one candidate"))?
                // Return the index of the largest sample.
                .map(|v| v.0)
        }),
        input_metric.clone(),
        output_measure.clone(),
        PrivacyMap::new_fallible(move |d_in: &TIA| {
            // convert L_\infty distance to range distance
            let d_in = input_metric.range_distance(d_in.clone())?;

            // convert data type to f64
            let d_in = f64::inf_cast(d_in)?;

            // upper bound the privacy loss in terms of the output measure
            output_measure.privacy_map(d_in, scale)
        }),
    )
}

/// # Proof Definition
/// Defines the noise distribution associated with the privacy measure.
pub trait SelectionMeasure: 'static + Measure<Distance = f64> {
    type RV: InverseCDF;

    /// # Proof Definition
    /// `scale` must be non-negative.
    ///
    /// Returns a random variable.
    fn random_variable(shift: FBig, scale: FBig) -> Fallible<Self::RV>;

    /// # Proof Definition
    /// Given a mechanism that computes $\mathcal{M}(x) = \mathrm{argmax}_i z_i$,
    /// where each $z_i \sim \mathrm{random\_variable}(\mathrm{shift}=x_i, \mathrm{scale}=\texttt{scale})$,
    ///
    /// the mechanism must satisfy `d_out = self.privacy_map(d_in)`,
    /// where `d_in` is the sensitivity in terms of the range distance
    ///
    /// ```math
    /// d_{\mathrm{Range}}(x, x') = \max_{ij} |(x_i - x'_i) - (x_j - x'_j)|,
    /// ```
    ///
    /// and `d_out` is the privacy loss parameter in terms of the output measure `Self`.
    fn privacy_map(&self, d_in: f64, scale: f64) -> Fallible<f64> {
        if d_in.is_sign_negative() {
            return fallible!(
                InvalidDistance,
                "sensitivity ({}) must be non-negative",
                d_in
            );
        }

        if scale.is_zero() {
            return Ok(f64::INFINITY);
        }

        // d_out >= d_in / scale
        d_in.inf_div(&scale)
    }
}

impl SelectionMeasure for RangeDivergence {
    type RV = GumbelRV;

    fn random_variable(shift: FBig, scale: FBig) -> Fallible<Self::RV> {
        GumbelRV::new(shift, scale)
    }
}

impl SelectionMeasure for MaxDivergence {
    type RV = ExponentialRV;

    fn random_variable(shift: FBig, scale: FBig) -> Fallible<Self::RV> {
        ExponentialRV::new(shift, scale)
    }
}

#[derive(PartialEq, Clone, Copy, Debug)]
#[cfg_attr(feature = "polars", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "polars", serde(rename_all = "lowercase"))]
pub enum Optimize {
    Min,
    Max,
}

impl Display for Optimize {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Optimize::Min => f.write_str("min"),
            Optimize::Max => f.write_str("max"),
        }
    }
}

impl TryFrom<&str> for Optimize {
    type Error = crate::error::Error;
    fn try_from(s: &str) -> Fallible<Self> {
        Ok(match s {
            "min" => Optimize::Min,
            "max" => Optimize::Max,
            _ => return fallible!(FailedCast, "optimize must be \"min\" or \"max\""),
        })
    }
}
