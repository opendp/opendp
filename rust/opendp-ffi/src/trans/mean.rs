use std::convert::TryFrom;
use std::iter::Sum;
use std::ops::Sub;
use std::os::raw::{c_char, c_uint, c_void};

use num::Float;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{BoundedMeanConstant, make_bounded_mean};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_bounded_mean(
    lower: *const c_void, upper: *const c_void, n: c_uint,
    MI: *const c_char, MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<T>(
        lower: *const c_void, upper: *const c_void, n: usize,
        MI: Type, MO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where T: DistanceConstant + Sub<Output=T> + Float,
              for<'a> T: Sum<&'a T> {
        fn monomorphize2<MI, MO>(lower: MO::Distance, upper: MO::Distance, n: usize) -> FfiResult<*mut AnyTransformation>
            where MI: 'static + DatasetMetric<Distance=u32>,
                  MO: 'static + SensitivityMetric,
                  MO::Distance: DistanceConstant + Sub<Output=MO::Distance> + Float,
                  for<'a> MO::Distance: Sum<&'a MO::Distance>,
                  (MI, MO): BoundedMeanConstant<MI, MO> {
            make_bounded_mean::<MI, MO>(lower, upper, n).into_any()
        }
        let lower = *try_as_ref!(lower as *const T);
        let upper = *try_as_ref!(upper as *const T);
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<T>, L2Sensitivity<T>])
        ], (lower, upper, n))
    }
    let n = n as usize;
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(T, @floats)], (lower, upper, n, MI, MO))
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
    fn test_make_bounded_sum_n() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_bounded_mean(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            3 as c_uint,
            "SymmetricDistance".to_char_p(),
            "L2Sensitivity<f64>".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![1.0, 2.0, 3.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 2.0);
        Ok(())
    }
}
