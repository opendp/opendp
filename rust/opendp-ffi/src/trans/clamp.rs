use std::convert::TryFrom;
use std::ops::{Sub, Bound};
use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, SensitivityMetric, Domain};
use opendp::dist::{HammingDistance, SymmetricDistance, AbsoluteDistance, L1Distance, L2Distance};
use opendp::dom::{AllDomain, VectorDomain, IntervalDomain};
use opendp::err;
use opendp::traits::{DistanceCast, DistanceConstant};
use opendp::trans::{ClampableDomain, make_clamp, make_unclamp, UnclampableDomain};

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
            (M, [AbsoluteDistance<Q>]),
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


#[no_mangle]
pub extern "C" fn opendp_trans__make_unclamp(
    lower: *const c_void, upper: *const c_void,
    M: *const c_char, T: *const c_char,
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize_dataset<M, T>(
        lower: *const c_void, upper: *const c_void
    ) -> FfiResult<*mut AnyTransformation>
        where VectorDomain<IntervalDomain<T>>: UnclampableDomain<Atom=T>,
              <VectorDomain<IntervalDomain<T>> as Domain>::Carrier: Clone,
              M: 'static + DatasetMetric,
              T: 'static + Clone + PartialOrd {
        let lower = try_as_ref!(lower as *const T).clone();
        let upper = try_as_ref!(upper as *const T).clone();
        make_unclamp::<VectorDomain<IntervalDomain<T>>, M>(
            Bound::Included(lower), Bound::Included(upper),
        ).into_any()
    }

    fn monomorphize_sensitivity<Q: DistanceConstant + One>(
        lower: *const c_void, upper: *const c_void, M: Type, T: Type,
    ) -> FfiResult<*mut AnyTransformation> {

        fn monomorphize_sensitivity_2<M, T>(
            lower: *const c_void, upper: *const c_void,
        ) -> FfiResult<*mut AnyTransformation>
            where IntervalDomain<T>: UnclampableDomain<Atom=T>,
                  <IntervalDomain<T> as Domain>::Carrier: Clone,
                  M: 'static + SensitivityMetric,
                  M::Distance: DistanceConstant + One,
                  T: 'static + Clone + PartialOrd + DistanceCast + Sub<Output=T> {

            let lower = try_as_ref!(lower as *const T).clone();
            let upper = try_as_ref!(upper as *const T).clone();
            make_unclamp::<IntervalDomain<T>, M>(
                Bound::Included(lower), Bound::Included(upper),
            ).into_any()
        }
        dispatch!(monomorphize_sensitivity_2, [
            (M, [AbsoluteDistance<Q>, L1Distance<Q>, L2Distance<Q>]),
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
            "AbsoluteDistance<f64>".to_char_p(),
            "f64".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(-1.0);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: f64 = Fallible::from(res)?.downcast()?;
        assert_eq!(res, 0.0);
        Ok(())
    }

    #[test]
    fn test_make_vector_clamp() -> Fallible<()> {
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
