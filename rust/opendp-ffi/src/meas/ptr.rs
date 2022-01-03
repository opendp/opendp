use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::{c_char, c_void};

use num::Float;

use opendp::dist::IntDistance;
use opendp::err;
use opendp::meas::make_base_ptr;
use opendp::samplers::SampleLaplace;
use opendp::traits::{CheckNull, InfCast};

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::Type;

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
        where TK: 'static + Eq + Hash + Clone + CheckNull,
              TV: 'static + Float + CheckNull + InfCast<IntDistance> + SampleLaplace {
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
