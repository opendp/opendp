use std::convert::TryFrom;
use std::hash::Hash;
use std::ops::AddAssign;
use std::os::raw::{c_char, c_uint};

use num::{Bounded, Integer, One, Zero};
use num::traits::FloatConst;

use opendp::core::{SensitivityMetric};
use opendp::dist::{L1Distance, L2Distance};
use opendp::err;
use opendp::traits::DistanceConstant;
use opendp::trans::{CountByConstant, make_count, make_count_by, make_count_by_categories, make_count_distinct};

use crate::any::{AnyObject, AnyTransformation};
use crate::any::Downcast;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::util::Type;

#[no_mangle]
pub extern "C" fn opendp_trans__make_count(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static,
              TO: 'static + TryFrom<usize> + Bounded + One + DistanceConstant {
        make_count::<TIA, TO>().into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @primitives),
        (TO, @integers)
    ], ())
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count_distinct(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO: 'static>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Eq + Hash,
              TO: 'static + TryFrom<usize> + Bounded + One + DistanceConstant {
        make_count_distinct::<TIA, TO>().into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @hashable),
        (TO, @integers)
    ], ())
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by_categories(
    categories: *const AnyObject,
    MO: *const c_char, TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        categories: *const AnyObject,
        MO: Type, TI: Type, TO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MO, TI, TO>(categories: *const AnyObject) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant + One,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign {
            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TI>>()).clone();
            make_count_by_categories::<MO, TI, TO>(categories).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TI, @hashable),
            (TO, @integers)
        ], (categories))
    }
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (QO, @floats)
    ], (categories, MO, TI, TO))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by(
    n: c_uint,
    MO: *const c_char, TI: *const c_char, TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        n: usize, MO: Type, TI: Type, TO: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant + FloatConst + One {
        fn monomorphize2<MO, TI, TO>(n: usize) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant + FloatConst + One,
                  TI: 'static + Eq + Hash + Clone,
                  TO: 'static + Integer + Zero + One + AddAssign {
            make_count_by::<MO, TI, TO>(n).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TI, @hashable),
            (TO, @integers)
        ], (n))
    }
    let n = n as usize;
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(QO, @floats)], (n, MO, TI, TO))
}
