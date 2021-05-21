use std::convert::TryFrom;
use std::os::raw::{c_char, c_void};

use num::{CheckedAdd, CheckedSub, Float, Zero};

use opendp::err;
use opendp::meas::make_base_geometric;
use opendp::samplers::SampleGeometric;
use opendp::traits::DistanceCast;

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_simple_geometric(
    scale: *const c_void, min: *const c_void, max: *const c_void,
    T: *const c_char, QO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<T, QO>(
        scale: *const c_void, min: *const c_void, max: *const c_void,
    ) -> FfiResult<*mut AnyMeasurement>
        where T: 'static + Clone + SampleGeometric + CheckedSub<Output=T> + CheckedAdd<Output=T> + DistanceCast + Zero + PartialOrd,
              QO: 'static + Float + DistanceCast, f64: From<QO> {
        let scale = *try_as_ref!(scale as *const QO);
        let min = try_as_ref!(min as *const T).clone();
        let max = try_as_ref!(max as *const T).clone();
        make_base_geometric::<T, QO>(scale, min, max).into_any()
    }
    let T = try_!(Type::try_from(T));
    let QO = try_!(Type::try_from(QO));
    dispatch!(monomorphize, [
        (T, @integers),
        (QO, @floats)
    ], (scale, min, max))
}


#[cfg(test)]
mod tests {
    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;

    use super::*;

    #[test]
    fn test_make_base_simple_geometric() -> Fallible<()> {
        let measurement = Result::from(opendp_meas__make_base_simple_geometric(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(0) as *const c_void,
            util::into_raw(100) as *const c_void,
            "i32".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(99);
        let res = core::opendp_core__measurement_invoke(&measurement, arg);
        let res: i32 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 99);
        Ok(())
    }
}
