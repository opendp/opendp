use std::convert::TryFrom;
use std::ops::Sub;
use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, SymmetricDistance, L1Sensitivity, L2Sensitivity};
use opendp::dom::{AllDomain, VectorDomain};
use opendp::err;
use opendp::traits::{DistanceCast, DistanceConstant};
use opendp::trans::{ClampableDomain, make_clamp};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{MetricClass, Type};

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize_dataset<M, T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
        where VectorDomain<AllDomain<T>>: ClampableDomain<M, Atom=T>,
              M: 'static + DatasetMetric,
              T: 'static + Clone + PartialOrd {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_clamp::<VectorDomain<AllDomain<T>>, M>(
            lower, upper,
        ).into_any()
    }

    fn monomorphize_sensitivity<Q: DistanceConstant + One>(
        lower: *const c_void, upper: *const c_void, M: Type, T: Type,
    ) -> FfiResult<*mut AnyTransformation> {
        fn monomorphize_sensitivity_2<M, T>(
            lower: *const c_void, upper: *const c_void,
        ) -> FfiResult<*mut AnyTransformation>
            where AllDomain<T>: ClampableDomain<M, Atom=T>,
                  M: 'static + SensitivityMetric,
                  M::Distance: DistanceConstant + One,
                  T: 'static + Clone + PartialOrd + DistanceCast + Sub<Output=T> {
            let lower = try_as_ref!(lower as *const T).clone();
            let upper = try_as_ref!(upper as *const T).clone();
            make_clamp::<AllDomain<T>, M>(
                lower, upper,
            ).into_any()
        }
        dispatch!(monomorphize_sensitivity_2, [
            (M, [L1Sensitivity<Q>, L2Sensitivity<Q>]),
            (T, @numbers)
        ], (lower, upper))
    }

    let M = try_!(Type::try_from(M));
    let T = try_!(Type::try_from(T));
    match try_!(M.get_metric_class()) {
        MetricClass::Dataset =>
            dispatch!(monomorphize_dataset, [(M, @dist_dataset), (T, @numbers)], (lower, upper)),
        MetricClass::Sensitivity => {
            let Q = try_!(M.get_sensitivity_distance());
            dispatch!(monomorphize_sensitivity, [(Q, @numbers)], (lower, upper, M, T))
        }
    }
}


#[cfg(test)]
mod tests {
    use std::os::raw::c_void;

    use opendp::error::Fallible;

    use crate::any::{AnyObject, Downcast};
    use crate::core;
    use crate::util;
    use crate::util::ToCharP;
    use crate::trans::clamp::opendp_trans__make_clamp;

    #[test]
    fn test_make_clamp_sensitivity() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_clamp(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "L2Sensitivity<f64>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(-1.0);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 0.0);
        Ok(())
    }

    #[test]
    fn test_make_clamp_vec() -> Fallible<()> {
        let transformation = Result::from(opendp_trans__make_clamp(
            util::into_raw(0.0) as *const c_void,
            util::into_raw(10.0) as *const c_void,
            "SymmetricDistance".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }

}
