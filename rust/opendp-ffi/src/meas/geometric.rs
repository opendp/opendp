use std::os::raw::{c_char, c_void};

use num::{CheckedAdd, CheckedSub, Float, Zero};

use opendp::err;
use opendp::meas::make_base_geometric;
use opendp::samplers::SampleGeometric;
use opendp::traits::DistanceCast;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::Type;
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_simple_geometric(
    scale: *const c_void, min: *const c_void, max: *const c_void,
    T: *const c_char, QO: *const c_char
) -> FfiResult<*mut FfiMeasurement> {

    fn monomorphize<T, QO>(
        scale: *const c_void, min: *const c_void, max: *const c_void
    ) -> FfiResult<*mut FfiMeasurement>
        where T: 'static + Clone + SampleGeometric + CheckedSub<Output=T> + CheckedAdd<Output=T> + DistanceCast + Zero,
              QO: 'static + Float + DistanceCast, f64: From<QO> {
        let scale = *try_as_ref!(scale as *const QO);
        let min = try_as_ref!(min as *const T).clone();
        let max = try_as_ref!(max as *const T).clone();
        make_base_geometric::<T, QO>(scale, min, max).into()
    }
    let T = try_!(Type::try_from(T));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (scale, min, max))
}