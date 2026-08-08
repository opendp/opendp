use crate::{
    core::{Domain, Measure, Measurement, Metric, MetricSpace, PrivacyMap},
    error::Fallible,
    measures::{Approximate, MultiDP, PrivacyGuarantee, zCDP},
};

#[cfg(feature = "ffi")]
mod ffi;

#[cfg(test)]
mod test;

/// Constructs a new output measurement where the output measure
/// is cast from `zCDP` to `MultiDP`.
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be cast
///
/// # Generics
/// * `DI` - Input Domain
/// * `TO` - Output Type
/// * `MI` - Input Metric
/// * `MO` - Privacy Measure
#[allow(non_snake_case)]
pub fn make_zCDP_to_multiDP<DI, MI, MO, TO>(
    meas: Measurement<DI, MI, MO, TO>,
) -> Fallible<Measurement<DI, MI, MO::ApproxMeasure, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + ConcentratedMeasure,
    (DI, MI): MetricSpace,
{
    let privacy_map = meas.privacy_map.clone();
    Measurement::new(
        meas.input_domain.clone(),
        meas.input_metric.clone(),
        MO::ApproxMeasure::default(),
        meas.function.clone(),
        PrivacyMap::new_fallible(move |d_in: &MI::Distance| {
            let d_mid = privacy_map.eval(d_in)?;
            MO::convert(d_mid)
        }),
    )
}

#[deprecated(since = "0.15.0", note = "Use `make_zCDP_to_multiDP` instead.")]
/// Deprecated compatibility alias for [`make_zCDP_to_multiDP`].
///
/// # Arguments
/// * `meas` - a measurement with a privacy measure to be cast
#[allow(non_snake_case)]
pub fn make_zCDP_to_approxDP<DI, MI, MO, TO>(
    meas: Measurement<DI, MI, MO, TO>,
) -> Fallible<Measurement<DI, MI, MO::ApproxMeasure, TO>>
where
    DI: Domain,
    MI: 'static + Metric,
    MO: 'static + ConcentratedMeasure,
    (DI, MI): MetricSpace,
{
    make_zCDP_to_multiDP(meas)
}

pub trait ConcentratedMeasure: Measure {
    type ApproxMeasure: Measure;

    fn convert(d_mid: Self::Distance) -> Fallible<<Self::ApproxMeasure as Measure>::Distance>;
}

impl ConcentratedMeasure for zCDP {
    type ApproxMeasure = MultiDP;

    fn convert(rho: Self::Distance) -> Fallible<<Self::ApproxMeasure as Measure>::Distance> {
        PrivacyGuarantee::new().with_zCDP(rho, 0.0)
    }
}

impl ConcentratedMeasure for Approximate<zCDP> {
    type ApproxMeasure = Approximate<MultiDP>;

    fn convert(
        (rho, delta): Self::Distance,
    ) -> Fallible<<Self::ApproxMeasure as Measure>::Distance> {
        // The source delta belongs to the approximate-zCDP statement. The
        // target's separate approximate-DP relaxation remains zero.
        Ok((PrivacyGuarantee::new().with_zCDP(rho, delta)?, 0.0))
    }
}
