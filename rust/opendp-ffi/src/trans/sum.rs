use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::{Abs, DistanceConstant};
use opendp::trans::{BoundedSumConstant, make_bounded_sum, make_bounded_sum_n};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum(
    lower: *const c_void, upper: *const c_void,
    MI: *const c_char, MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        lower: *const c_void, upper: *const c_void,
        MI: Type, MO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where for<'a> T: DistanceConstant + Sub<Output=T> + Abs + Sum<&'a T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance) -> FfiResult<*mut AnyTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  for<'a> MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Abs + Sum<&'a MO::Distance>,
                  (MI, MO): BoundedSumConstant<MI, MO> {
            make_bounded_sum::<MI, MO>(lower, upper).into_any()
        }
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper))
    }
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(T, @numbers)], (lower, upper, MI, MO))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_sum_n(
    lower: *const c_void, upper: *const c_void, n: c_uint,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        lower: *const c_void, upper: *const c_void, n: usize,
        MO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant + Sub<Output=T>,
              for<'a> T: Sum<&'a T> {
        fn monomorphize2<MO>(lower: MO::Distance, upper: MO::Distance, n: usize) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance>,
                  for<'a> MO::Distance: Sum<&'a MO::Distance> {
            make_bounded_sum_n::<MO>(lower, upper, n).into_any()
        }
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        dispatch!(monomorphize2, [
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, n))
    }
    let n = n as usize;

    let MO = try_!(Type::try_from(MO));
    let TO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(TO, @numbers)], (lower, upper, n, MO))
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
    fn test_make_bounded_sum() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_sum(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "SymmetricDistance".to_char_p(),
            "L2Sensitivity<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }

    #[test]
    fn test_make_bounded_sum_n() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_sum_n(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            3 as c_uint,
            "L2Sensitivity<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 6.0);
        Ok(())
    }
}
