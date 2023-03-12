use std::ffi::c_char;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    ffi::{any::{AnyObject, Downcast, AnyMeasurement}, util::{Type, to_str}},
    measurements::{make_base_discrete_exponential, Optimize},
    traits::{samplers::SampleUniform, CheckNull, Float, InfCast, Number, RoundCast},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_exponential(
    temperature: *const AnyObject,
    optimize: *const c_char,
    TIA: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TIA, QO>(temperature: *const AnyObject, optimize: Optimize) -> FfiResult<*mut AnyMeasurement>
    where
        TIA: Clone + CheckNull + Number,
        QO: 'static + InfCast<TIA> + RoundCast<TIA> + Float + SampleUniform,
    {
        let temperature = *try_!(try_as_ref!(temperature).downcast_ref::<QO>());
        make_base_discrete_exponential::<TIA, QO>(temperature, optimize).into_any()
    }
    let optimize = match try_!(to_str(optimize)) {
        i if i.to_lowercase().starts_with("min") => Optimize::Min,
        i if i.to_lowercase().starts_with("max") => Optimize::Max,
        _ => return err!(FFI, "optimize must start with \"min\" or \"max\"").into()
    };
    let TIA = try_!(Type::try_from(TIA));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (TIA, [u32, u64, i32, i64, usize, f32, f64]),
        (QO, @floats)
    ], (temperature, optimize))
}
