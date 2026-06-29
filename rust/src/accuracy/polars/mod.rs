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

pub use crate::polars::accuracy::summarize_lazyframe;

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
pub fn summarize_polars_measurement<MI: Metric, MO: 'static + Measure>(
    measurement: Measurement<LazyFrameDomain, MI, MO, OnceFrame>,
    alpha: Option<f64>,
) -> Fallible<DataFrame>
where
    (LazyFrameDomain, MI): MetricSpace,
{
    crate::polars::accuracy::summarize_polars_measurement(measurement, alpha)
}
