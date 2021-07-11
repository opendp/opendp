use std::convert::TryFrom;
use std::ops::{Bound};
use std::os::raw::{c_char, c_void};

use num::One;

use opendp::core::{DatasetMetric, Metric};
use opendp::dist::{SubstituteDistance, SymmetricDistance, AbsoluteDistance, L1Distance, L2Distance, IntDistance};
use opendp::dom::{AllDomain, VectorDomain, IntervalDomain};
use opendp::err;
use opendp::traits::{DistanceConstant, InfCast};
use opendp::trans::{ClampableDomain, make_clamp, make_unclamp, UnclampableDomain};

use crate::any::AnyTransformation;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::{MetricClass, Type};

#[no_mangle]
pub extern "C" fn opendp_trans__make_clamp(
    lower: *const c_void, upper: *const c_void,
    DI: *const c_char, M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DI = try_!(Type::try_from(DI));
    let DIA = try_!(DI.get_domain_atom());
    let M = try_!(Type::try_from(M));

    // if MetricClass::Dataset = M.get_metric_class() {
    //     return in here
    // }
    //
    // if MetricClass::Senstivity = M.get_metric_class() {
    //     return in here
    // }
    // unreachable!()



    match try_!(M.get_metric_class()) {
        MetricClass::Dataset => {
            fn monomorphize_dataset<T>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
                where VectorDomain<AllDomain<T>>: ClampableDomain<SymmetricDistance, Atom=T>,
                      T: 'static + Clone + PartialOrd {
                let lower = try_as_ref!(lower as *const T).clone();
                let upper = try_as_ref!(upper as *const T).clone();
                make_clamp::<VectorDomain<AllDomain<T>>, SymmetricDistance>(lower, upper).into_any()
            }
            dispatch!(monomorphize_dataset, [
                (DIA, @numbers)
            ], (lower, upper))
        },

        MetricClass::Sensitivity => {
            fn monomorphize_sensitivity<T, Q>(
                lower: *const c_void, upper: *const c_void
            ) -> FfiResult<*mut AnyTransformation>
                where AllDomain<T>: ClampableDomain<AbsoluteDistance<Q>, Atom=T>,
                      Q: DistanceConstant<IntDistance> + One,
                      T: 'static + Clone + PartialOrd,
                      IntDistance: InfCast<Q> {
                let lower = try_as_ref!(lower as *const T).clone();
                let upper = try_as_ref!(upper as *const T).clone();
                make_clamp::<AllDomain<T>, AbsoluteDistance<Q>>(lower, upper).into_any()
            }
            let Q = try_!(M.get_sensitivity_distance());
            dispatch!(monomorphize_sensitivity, [
                (DIA, @numbers),
                (Q, @numbers)
            ], (lower, upper))
        }
    }
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_unclamp(
    lower: *const c_void, upper: *const c_void,
    DI: *const c_char, M: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DI = try_!(Type::try_from(DI));
    let M = try_!(Type::try_from(M));

    let T = try_!(DI.get_domain_atom());

    match try_!(M.get_metric_class()) {
        MetricClass::Dataset => {
            fn monomorphize_dataset<T, M>(lower: *const c_void, upper: *const c_void) -> FfiResult<*mut AnyTransformation>
                where VectorDomain<IntervalDomain<T>>: UnclampableDomain<Atom=T, Carrier=Vec<T>>,
                      T: 'static + Clone + PartialOrd,
                      M: 'static + DatasetMetric {
                let lower = try_as_ref!(lower as *const T).clone();
                let upper = try_as_ref!(upper as *const T).clone();
                make_unclamp::<VectorDomain<IntervalDomain<T>>, M>(
                    Bound::Included(lower), Bound::Included(upper)
                ).into_any()
            }
            dispatch!(monomorphize_dataset, [
                (T, @numbers),
                (M, @dist_dataset)
            ], (lower, upper))
        },

        MetricClass::Sensitivity => {
            fn monomorphize_sensitivity<T, Q>(
                lower: *const c_void, upper: *const c_void, DI: Type, M: Type
            ) -> FfiResult<*mut AnyTransformation>
                where IntervalDomain<T>: UnclampableDomain<Atom=T, Carrier=T>,
                      Q: DistanceConstant<Q> + One,
                      T: 'static + Clone + PartialOrd {
                fn monomorphize_sensitivity_2<DI, M>(
                    lower: DI::Atom, upper: DI::Atom,
                ) -> FfiResult<*mut AnyTransformation>
                    where DI: 'static + UnclampableDomain,
                          DI::Carrier: Clone,
                          M: 'static + Metric,
                          DI::Atom: 'static + Clone + PartialOrd,
                          M::Distance: DistanceConstant<M::Distance> + One {
                    make_unclamp::<DI, M>(
                        Bound::Included(lower), Bound::Included(upper),
                    ).into_any()
                }
                let lower = try_as_ref!(lower as *const T).clone();
                let upper = try_as_ref!(upper as *const T).clone();

                dispatch!(monomorphize_sensitivity_2, [
                    (DI, [IntervalDomain<T>]),
                    (M, [AbsoluteDistance<Q>, L1Distance<Q>, L2Distance<Q>])
                ], (lower, upper))
            }
            let Q = try_!(M.get_sensitivity_distance());
            dispatch!(monomorphize_sensitivity, [
                (T, @numbers),
                (Q, @numbers)
            ], (lower, upper, DI, M))
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
            "AllDomain<f64>".to_char_p(),
            "AbsoluteDistance<f64>".to_char_p(),
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
            "VectorDomain<AllDomain<f64>>".to_char_p(),
            "SymmetricDistance".to_char_p(),
        ))?;
        let arg = AnyObject::new_raw(vec![-1.0, 5.0, 11.0]);
        let res = core::opendp_core__transformation_invoke(&transformation, arg);
        let res: Vec<f64> = Fallible::from(res)?.downcast()?;
        assert_eq!(res, vec![0.0, 5.0, 10.0]);
        Ok(())
    }

}
