use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::{c_char, c_uint};

use num::{Integer, One, Zero, Bounded};
use num::traits::FloatConst;

use opendp::core::{DatasetMetric, SensitivityMetric};
use opendp::dist::{HammingDistance, L1Sensitivity, L2Sensitivity, SymmetricDistance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{CountByConstant, make_count, make_count_by, make_count_by_categories};

use crate::any::{AnyObject, AnyTransformation};
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;
use crate::any::Downcast;

#[no_mangle]
pub extern "C" fn opendp_trans__make_count(
    MI: *const c_char,
    MO: *const c_char,
    T: *const c_char
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<QO: TryFrom<usize> + Bounded + One + DistanceConstant>(
        MI: Type, MO: Type, T: Type
    ) -> FfiResult<*mut AnyTransformation> {
        fn monomorphize2<MI, MO, T: 'static>() -> FfiResult<*mut AnyTransformation>
            where MI: 'static + DatasetMetric + Clone,
                  MO: 'static + SensitivityMetric + Clone {
            make_count::<MI, MO, T>().into_any()
        }
        dispatch!(monomorphize2, [
            (MI, [SymmetricDistance, HammingDistance]),
            (MO, [L1Sensitivity<QO>, L2Sensitivity<QO>]),
            (T, @primitives)
        ], ())
    }
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let T = try_!(Type::try_from(T));
    let QO = try_!(MO.get_sensitivity_distance());

    dispatch!(monomorphize, [
        (QO, @integers)
    ], (MI, MO, T))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by_categories(
    categories: *const AnyObject,
    MI: *const c_char, MO: *const c_char,
    TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        categories: *const AnyObject,
        MI: Type, MO: Type, TI: Type, TO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MI, MO, TI, TO>(categories: *const AnyObject) -> FfiResult<*mut AnyTransformation>
            where MI: 'static + DatasetMetric,
                  MO: 'static + SensitivityMetric,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign,
                  MO::Distance: DistanceConstant + FloatConst + One,
                  (MI, MO): CountByConstant<MI, MO> {
            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TI>>()).clone();
            make_count_by_categories::<MI, MO, TI, TO>(categories).into_any()
        }
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<QO>, L2Sensitivity<QO>]),
            (TI, @hashable),
            (TO, @integers)
        ], (categories))
    }
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (QO, @floats)
    ], (categories, MI, MO, TI, TO))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by(
    n: c_uint,
    MI: *const c_char, MO: *const c_char,
    TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        n: usize, MI: Type, MO: Type, TI: Type, TO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MI, MO, TI, TO>(n: usize) -> FfiResult<*mut AnyTransformation>
            where MI: 'static + DatasetMetric,
                  MO: 'static + SensitivityMetric,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign,
                  MO::Distance: DistanceConstant + FloatConst + One,
                  (MI, MO): CountByConstant<MI, MO> {
            make_count_by::<MI, MO, TI, TO>(n).into_any()
        }
        dispatch!(monomorphize2, [
            (MI, [HammingDistance, SymmetricDistance]),
            (MO, [L1Sensitivity<QO>, L2Sensitivity<QO>]),
            (TI, @hashable),
            (TO, @integers)
        ], (n))
    }
    let n = n as usize;
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(QO, @floats)], (n, MI, MO, TI, TO))
}
