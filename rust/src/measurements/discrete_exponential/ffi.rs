use std::ffi::c_char;

use crate::{
    core::{FfiResult, IntoAnyMeasurementFfiResultExt},
    ffi::{any::{AnyObject, Downcast, AnyMeasurement}, util::Type},
    measurements::make_base_discrete_exponential,
    traits::{samplers::SampleUniform, CheckNull, Float, InfCast, Number, RoundCast},
};

#[no_mangle]
pub extern "C" fn opendp_measurements__make_base_discrete_exponential(
    temperature: *const AnyObject,
    TIA: *const c_char,
    QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TIA, QO>(temperature: *const AnyObject) -> FfiResult<*mut AnyMeasurement>
    where
        TIA: Clone + CheckNull + Number,
        QO: 'static + InfCast<TIA> + RoundCast<TIA> + Float + SampleUniform,
    {
        let temperature = *try_!(try_as_ref!(temperature).downcast_ref::<QO>());
        make_base_discrete_exponential::<TIA, QO>(temperature).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (TIA, @numbers),
        (QO, @floats)
    ], (temperature))
}
