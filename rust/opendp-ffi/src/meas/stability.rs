use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::{c_char, c_void};

use num::{Float, Integer, NumCast, One, Zero};

use opendp::err;
use opendp::core::SensitivityMetric;
use opendp::dist::{L1Sensitivity, L2Sensitivity};
use opendp::meas::{BaseStabilityNoise, make_base_stability};
use opendp::samplers::CastRug;

use crate::core::{FfiMeasurement, FfiResult};
use crate::util::Type;
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_stability(
    n: usize,
    scale: *const c_void,
    threshold: *const c_void,
    MI: *const c_char,  // input metric space (sensitivity)
    TIK: *const c_char,  // type of input key (hashable)
    TIC: *const c_char,  // type of input count (int)
) -> FfiResult<*mut FfiMeasurement> {

    fn monomorphize<TOC>(
        n: usize, scale: *const c_void, threshold: *const c_void,
        MI: Type, TIK: Type, TIC: Type
    ) -> FfiResult<*mut FfiMeasurement>
        where TOC: 'static + PartialOrd + Clone + NumCast + Float + CastRug {

        fn monomorphize2<MI, TIK, TIC>(
            n: usize, scale: MI::Distance, threshold: MI::Distance
        ) -> FfiResult<*mut FfiMeasurement>
            where MI: 'static + SensitivityMetric + BaseStabilityNoise,
                  TIK: 'static + Eq + Hash + Clone,
                  TIC: 'static + Integer + Zero + One + AddAssign + Clone + NumCast,
                  MI::Distance: 'static + Clone + NumCast + PartialOrd + Float + CastRug {
            make_base_stability::<MI, TIK, TIC>(n, scale, threshold).into()
        }
        let scale = *try_as_ref!(scale as *const TOC);
        let threshold = *try_as_ref!(threshold as *const TOC);
        dispatch!(monomorphize2, [
            (MI, [L1Sensitivity<TOC>, L2Sensitivity<TOC>]),
            (TIK, @hashable),
            (TIC, @integers)
        ], (n, scale, threshold))
    }
    let MI = try_!(Type::try_from(MI));
    let TIK = try_!(Type::try_from(TIK));
    let TIC = try_!(Type::try_from(TIC));

    let TOC = try_!(MI.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (TOC, @floats)
    ], (n, scale, threshold, MI, TIK, TIC))
}
