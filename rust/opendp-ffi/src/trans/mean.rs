use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use num::Float;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedMeanConstant, make_bounded_mean};

use crate::core::{FfiResult, FfiTransformation};
use crate::util::Type;
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_mean(
    lower: *const c_void, upper: *const c_void, length: c_uint,
    MI: *const c_char, MO: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize<T>(
        lower: *const c_void, upper: *const c_void, length: usize,
        MI: Type, MO: Type
    ) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float,
              for<'a> T: Sum<&'a T> {

        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance, length: usize) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Float,
                  for<'a> MO::Distance: Sum<&'a MO::Distance>,
                  (MI, MO): BoundedMeanConstant<MI, MO> {
            make_bounded_mean::<MI, MO>(lower, upper, length).into()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length))
    }
    let length = length as usize;
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(T, @floats)], (lower, upper, length, MI, MO))
}