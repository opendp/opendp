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
use crate::util::{parse_type_args, Type};

#[no_mangle]
pub extern "C" fn opendp_meas__make_base_stability(type_args: *const c_char, n: usize, scale: *const c_void, threshold: *const c_void) -> FfiResult<*mut FfiMeasurement> {
    fn monomorphize<TIC, TOC>(type_args: Vec<Type>, n: usize, scale: *const c_void, threshold: *const c_void) -> FfiResult<*mut FfiMeasurement>
        where TIC: 'static + Integer + Zero + One + AddAssign + Clone + NumCast,
              TOC: 'static + PartialOrd + Clone + NumCast + Float + CastRug {
        fn monomorphize2<MI, TIK, TIC>(n: usize, scale: MI::Distance, threshold: MI::Distance) -> FfiResult<*mut FfiMeasurement>
            where MI: 'static + SensitivityMetric + BaseStabilityNoise,
                  TIK: 'static + Eq + Hash + Clone,
                  TIC: 'static + Integer + Zero + One + AddAssign + Clone + NumCast,
                  MI::Distance: 'static + Clone + NumCast + PartialOrd + Float + CastRug {
            make_base_stability::<MI, TIK, TIC>(n, scale, threshold).into()
        }
        let scale = *try_as_ref!(scale as *const TOC);
        let threshold = *try_as_ref!(threshold as *const TOC);
        dispatch!(monomorphize2, [
            (type_args[0], [L1Sensitivity<TOC>, L2Sensitivity<TOC>]),
            (type_args[1], @hashable),
            (type_args[2], [TIC])
        ], (n, scale, threshold))
    }
    let type_args = try_!(parse_type_args(type_args, 4));
    dispatch!(monomorphize, [
        (type_args[2], @integers),
        (type_args[3], @floats)
    ], (type_args, n, scale, threshold))
}
