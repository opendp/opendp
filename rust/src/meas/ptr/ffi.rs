use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::err;
use crate::ffi::any::AnyMeasurement;
use crate::ffi::util::Type;
use crate::meas::{make_base_ptr};
use crate::traits::{Float, Hashable};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_ptr(
    scale: *const c_void,
    threshold: *const c_void,
    TK: *const c_char,  // atomic type of input key (hashable)
    TV: *const c_char,  // type of count (float)
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TK, TV>(
        scale: *const c_void, threshold: *const c_void
    ) -> FfiResult<*mut AnyMeasurement>
        where TK: Hashable,
              TV: Float {
        let scale = *try_as_ref!(scale as *const TV);
        let threshold = *try_as_ref!(threshold as *const TV);
        make_base_ptr::<TK, TV>(scale, threshold).into_any()
    }
    let TK = try_!(Type::try_from(TK));
    let TV = try_!(Type::try_from(TV));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TV, @floats)
    ], (scale, threshold))
}
