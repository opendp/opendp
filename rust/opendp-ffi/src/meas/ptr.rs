use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::{c_char, c_void};

use num::{Float, One, Zero};

use opendp::dist::IntDistance;
use opendp::err;
use opendp::meas::make_count_by_ptr;
use opendp::samplers::SampleLaplace;
use opendp::traits::{CheckNull, SaturatingAdd, InfCast};

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_meas__make_count_by_ptr(
    scale: *const c_void,
    threshold: *const c_void,
    TIA: *const c_char,  // atomic type of input key (hashable)
    TOC: *const c_char,  // type of count (float)
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TIA, TOC>(
        scale: *const c_void, threshold: *const c_void
    ) -> FfiResult<*mut AnyMeasurement>
        where TIA: 'static + Eq + Hash + Clone + CheckNull,
              TOC: 'static + Float + Zero + One + SaturatingAdd + CheckNull + InfCast<IntDistance> + SampleLaplace {
        let scale = *try_as_ref!(scale as *const TOC);
        let threshold = *try_as_ref!(threshold as *const TOC);
        make_count_by_ptr::<TIA, TOC>(scale, threshold).into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TOC = try_!(Type::try_from(TOC));

    dispatch!(monomorphize, [
        (TIA, @hashable),
        (TOC, @floats)
    ], (scale, threshold))
}
