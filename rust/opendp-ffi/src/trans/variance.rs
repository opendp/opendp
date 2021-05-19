use std::iter::Sum;
use std::ops::{Add, Div, Sub};
use std::os::raw::{c_char, c_uint};

use num::{Float, One, Zero};

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedCovarianceConstant, BoundedVarianceConstant, make_bounded_covariance, make_bounded_variance};

use crate::core::{FfiObject, FfiResult, FfiTransformation};
use crate::util::Type;
use std::convert::TryFrom;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_variance(
    lower: *const FfiObject, upper: *const FfiObject,
    length: c_uint, ddof: c_uint,
    MI: *const c_char, MO: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize<T>(
        lower: *const FfiObject, upper: *const FfiObject,
        length: usize, ddof: usize,
        MI: Type, MO: Type
    ) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float + for<'a> Sum<&'a T> + Sum<T>,
              for<'a> &'a T: Sub<Output=T> {

        fn monomorphize2<MI, MO>(
            lower: MO::Distance, upper: MO::Distance, length: usize, ddof: usize
        ) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Float + for<'a> Sum<&'a MO::Distance> + Sum<MO::Distance>,
                  for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
                  (MI, MO): BoundedVarianceConstant<MI, MO> {
            make_bounded_variance::<MI, MO>(lower, upper, length, ddof).into()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length, ddof))
    }
    let length = length as usize;
    let ddof = ddof as usize;

    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(TO, @floats)], (lower, upper, length, ddof, MI, MO))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_covariance(
    lower: *const FfiObject,
    upper: *const FfiObject,
    length: c_uint, ddof: c_uint,
    MI: *const c_char, MO: *const c_char
) -> FfiResult<*mut FfiTransformation> {

    fn monomorphize<T>(
        lower: *const FfiObject,
        upper: *const FfiObject,
        length: usize, ddof: usize,
        MI: Type, MO: Type
    ) -> FfiResult<*mut FfiTransformation>
        where T: DistanceConstant + Sub<Output=T> + Sum<T> + Zero + One,
              for<'a> T: Div<&'a T, Output=T> + Add<&'a T, Output=T>,
              for<'a> &'a T: Sub<Output=T> {

        fn monomorphize2<MI, MO>(
            lower: (MO::Distance, MO::Distance),
            upper: (MO::Distance, MO::Distance),
            length: usize, ddof: usize,
        ) -> FfiResult<*mut FfiTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Sum<MO::Distance> + Zero + One,
                  for<'a> MO::Distance: Div<&'a MO::Distance, Output=MO::Distance> + Add<&'a MO::Distance, Output=MO::Distance>,
                  for<'a> &'a MO::Distance: Sub<Output=MO::Distance>,
                  (MI, MO): BoundedCovarianceConstant<MI, MO> {
            make_bounded_covariance::<MI, MO>(lower, upper, length, ddof).into()
        }
        let lower = try_as_ref!(lower).as_ref::<(T, T)>().clone();
        let upper = try_as_ref!(upper).as_ref::<(T, T)>().clone();
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, length, ddof))
    }
    let length = length as usize;
    let ddof = ddof as usize;

    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(TO, @floats)], (lower, upper, length, ddof, MI, MO))
}
