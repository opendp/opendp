use std::ffi::c_char;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    domains::{AtomDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMeasurement, AnyMetric, AnyObject, Downcast},
        util::{to_str, Type},
    },
    measurements::{make_report_noisy_max_gumbel, Optimize},
    metrics::LInfDistance,
    traits::{
        samplers::{CastInternalRational, SampleUniform},
        CheckNull, Float, InfCast, Number, RoundCast,
    },
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_report_noisy_max_gumbel(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    scale: *const AnyObject,
    optimize: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA = try_!(input_domain.type_.get_atom());
    let scale = try_as_ref!(scale);

    let optimize = try_!(Optimize::try_from(try_!(to_str(optimize))));
    let QO = try_!(Type::try_from(QO));

    fn monomorphize<TIA, QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        scale: &AnyObject,
        optimize: Optimize,
    ) -> Fallible<AnyMeasurement>
    where
        TIA: Clone + CheckNull + Number + CastInternalRational,
        QO: 'static + InfCast<TIA> + RoundCast<TIA> + Float + SampleUniform + CastInternalRational,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<LInfDistance<TIA>>()?.clone();
        let scale = *scale.downcast_ref::<QO>()?;
        make_report_noisy_max_gumbel::<TIA, QO>(input_domain, input_metric, scale, optimize)
            .into_any()
    }

    dispatch!(monomorphize, [
        (TIA, [u32, u64, i32, i64, usize, f32, f64]),
        (QO, @floats)
    ], (input_domain, input_metric, scale, optimize))
    .into()
}
