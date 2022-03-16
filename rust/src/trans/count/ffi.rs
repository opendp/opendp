use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::c_char;

use num::{Bounded, Float, One, Zero};
use num::traits::FloatConst;

use crate::core::SensitivityMetric;
use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::dist::{IntDistance, L1Distance, L2Distance};
use crate::err;
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::any::Downcast;
use crate::ffi::util::Type;
use crate::traits::{CheckNull, DistanceConstant, ExactIntCast, InfCast, SaturatingAdd};
use crate::trans::{CountByCategoriesConstant, CountByConstant, make_count, make_count_by, make_count_by_categories, make_count_distinct};

#[no_mangle]
pub extern "C" fn opendp_trans__make_count(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + CheckNull,
              TO: 'static + ExactIntCast<usize> + Bounded + One + DistanceConstant<IntDistance> + CheckNull,
              IntDistance: InfCast<TO> {
        make_count::<TIA, TO>().into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @primitives),
        (TO, @numbers)
    ], ())
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_count_distinct(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO: 'static>() -> FfiResult<*mut AnyTransformation>
        where TIA: 'static + Eq + Hash + CheckNull,
              TO: 'static + ExactIntCast<usize> + Bounded + One + DistanceConstant<IntDistance> + CheckNull,
              IntDistance: InfCast<TO> {
        make_count_distinct::<TIA, TO>().into_any()
    }
    let TIA = try_!(Type::try_from(TIA));
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @hashable),
        (TO, @numbers)
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
        where QO: DistanceConstant<IntDistance> + One,
              IntDistance: InfCast<QO> {
        fn monomorphize2<MO, TI, TO>(categories: *const AnyObject) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByCategoriesConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + One,
                  TI: 'static + Eq + Hash + Clone + CheckNull,
                  TO: 'static + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {
            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TI>>()).clone();
            make_count_by_categories::<MO, TI, TO>(categories).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TI, @hashable),
            (TO, @numbers)
        ], (categories))
    }
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));

    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [
        (QO, @numbers)
    ], (categories, MO, TI, TO))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by(
    MO: *const c_char, TK: *const c_char, TV: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        MO: Type, TK: Type, TV: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant<IntDistance> + FloatConst + One + Float + ExactIntCast<usize>,
              IntDistance: InfCast<QO> {
        fn monomorphize2<MO, TK, TV>() -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + FloatConst + One,
                  TK: 'static + Eq + Hash + Clone + CheckNull,
                  TV: 'static + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {
            make_count_by::<MO, TK, TV>().into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TK, @hashable),
            (TV, @numbers)
        ], ())
    }
    let MO = try_!(Type::try_from(MO));
    let TK = try_!(Type::try_from(TK));
    let TV = try_!(Type::try_from(TV));

    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [(QO, @floats)], (MO, TK, TV))
}
