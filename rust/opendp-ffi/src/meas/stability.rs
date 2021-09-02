use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::{c_char, c_void};

use num::{Float, Integer, One, Zero};

use opendp::core::SensitivityMetric;
use opendp::dist::{L1Distance, L2Distance};
use opendp::err;
use opendp::meas::{BaseStabilityNoise, make_base_stability};
use opendp::samplers::CastInternalReal;
use opendp::traits::{ExactIntCast, CheckNull, TotalOrd};

use crate::any::AnyMeasurement;
use crate::core::{FfiResult, IntoAnyMeasurementFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_stability(
    size: usize,
    scale: *const c_void,
    threshold: *const c_void,
    MI: *const c_char,  // input metric (sensitivity)
    TIK: *const c_char,  // type of input key (hashable)
    TIC: *const c_char,  // type of input count (int)
) -> FfiResult<*mut AnyMeasurement> {
    fn monomorphize<TIC, TOC>(
        size: usize, scale: *const c_void, threshold: *const c_void,
        MI: Type, TIK: Type, TIC: Type,
    ) -> FfiResult<*mut AnyMeasurement>
        where TIC: 'static + Integer + Zero + One + AddAssign + Clone + CheckNull,
              TOC: 'static + TotalOrd + Clone + Float + CastInternalReal + ExactIntCast<usize> + ExactIntCast<TIC> + CheckNull {
        fn monomorphize2<MI, TIK, TIC>(
            size: usize, scale: MI::Distance, threshold: MI::Distance,
        ) -> FfiResult<*mut AnyMeasurement>
            where MI: 'static + SensitivityMetric + BaseStabilityNoise,
                  TIK: 'static + Eq + Hash + Clone + CheckNull,
                  TIC: 'static + Integer + Zero + One + AddAssign + Clone + CheckNull,
                  MI::Distance: 'static + Clone + TotalOrd + Float + CastInternalReal + ExactIntCast<usize> + ExactIntCast<TIC> + CheckNull {
            make_base_stability::<MI, TIK, TIC>(size, scale, threshold).into_any()
        }
        let scale = *try_as_ref!(scale as *const TOC);
        let threshold = *try_as_ref!(threshold as *const TOC);
        dispatch!(monomorphize2, [
            (MI, [L1Distance<TOC>, L2Distance<TOC>]),
            (TIK, @hashable),
            (TIC, [TIC])
        ], (size, scale, threshold))
    }
    let MI = try_!(Type::try_from(MI));
    let TIK = try_!(Type::try_from(TIK));
    let TIC = try_!(Type::try_from(TIC));

    let TOC = try_!(MI.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (TIC, @integers),
        (TOC, @floats)
    ], (size, scale, threshold, MI, TIK, TIC))
}
