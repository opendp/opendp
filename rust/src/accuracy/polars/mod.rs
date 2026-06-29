// Staging move of polars code into polars/.
// Keeping stub to maintain generated python as is
// TODO: Remove once migration and derive.rs updates are completed
use opendp_derive::bootstrap;
use polars::frame::DataFrame;

use crate::{
    core::{Measure, Measurement, Metric, MetricSpace},
    domains::LazyFrameDomain,
    error::Fallible,
    polars::OnceFrame,
};

#[bootstrap(
    name = "summarize_polars_measurement",
    features("contrib"),
    arguments(
        measurement(rust_type = "AnyMeasurement"),
        alpha(c_type = "AnyObject *", default = b"null")
    ),
    generics(MI(suppress), MO(suppress)),
    returns(c_type = "FfiResult<AnyObject *>")
)]
/// Summarize the statistics to be released from a measurement that returns a OnceFrame.
///
/// If a threshold is configured for censoring small/sensitive groups,
/// a threshold column will be included,
/// containing the cutoff for the respective count query being thresholded.
///
/// # Arguments
/// * `measurement` - computation from which you want to read noise scale parameters from
/// * `alpha` - optional statistical significance to use to compute accuracy estimates
pub fn summarize_polars_measurement<MI: Metric, MO: 'static + Measure>(
    measurement: Measurement<LazyFrameDomain, MI, MO, OnceFrame>,
    alpha: Option<f64>,
) -> Fallible<DataFrame>
where
    (LazyFrameDomain, MI): MetricSpace,
{
    crate::polars::accuracy::summarize_polars_measurement(measurement, alpha)
}
