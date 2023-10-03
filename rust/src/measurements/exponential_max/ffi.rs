use std::ffi::c_char;

use dashu::float::FBig;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, Downcast},
        util::to_str,
    },
    measurements::{make_report_noisy_max_exponential, Optimize},
    metrics::LInfDistance,
    traits::{InfCast, Number},
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_measurements__make_report_noisy_max_exponential(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: f64,
    optimize: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA = try_!(input_domain.type_.get_atom());

    let optimize = try_!(Optimize::try_from(try_!(to_str(optimize))));

    fn monomorphize<TIA>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: f64,
        optimize: Optimize,
    ) -> Fallible<AnyMeasurement>
    where
        TIA: Number,
        FBig: TryFrom<TIA> + TryFrom<f64>,
        f64: InfCast<TIA>,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<LInfDistance<TIA>>()?.clone();
        make_report_noisy_max_exponential::<TIA>(input_domain, input_metric, scale, optimize)
            .into_any()
    }

    dispatch!(
        monomorphize,
        [(TIA, [u32, u64, i32, i64, usize, f32, f64])],
        (input_domain, input_metric, scale, optimize)
    )
    .into()
}
