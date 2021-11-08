use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::{c_char, c_uint};

use num::{Bounded, Integer, One, Zero, Float};
use num::traits::FloatConst;

use opendp::core::{SensitivityMetric};
use opendp::dist::{L1Distance, L2Distance, IntDistance};
use opendp::err;
use opendp::traits::{DistanceConstant, InfCast, ExactIntCast, SaturatingAdd, CheckNull};
use opendp::trans::{CountByConstant, make_count, make_count_by, make_count_by_categories, make_count_distinct, CountByCategoriesConstant};

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
        where TIA: 'static + CheckNull,
              TO: 'static + ExactIntCast<usize> + Bounded + One + DistanceConstant<IntDistance> + CheckNull,
              IntDistance: InfCast<TO> {
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
        where TIA: 'static + Eq + Hash + CheckNull,
              TO: 'static + ExactIntCast<usize> + Bounded + One + DistanceConstant<IntDistance> + CheckNull,
              IntDistance: InfCast<TO> {
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
        where QO: DistanceConstant<IntDistance> + One,
              IntDistance: InfCast<QO> {
        fn monomorphize2<MO, TI, TO>(categories: *const AnyObject) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByCategoriesConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + One,
                  TI: 'static + Eq + Hash + Clone + CheckNull,
                  TO: 'static + Integer + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {
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

    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [
        (QO, @integers)
    ], (categories, MO, TI, TO))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_count_by(
    size: c_uint,
    MO: *const c_char, TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        size: usize, MO: Type, TIA: Type, TOA: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant<IntDistance> + FloatConst + One + Float + ExactIntCast<usize>,
              IntDistance: InfCast<QO> {
        fn monomorphize2<MO, TIA, TOA>(size: usize) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + FloatConst + One,
                  TIA: 'static + Eq + Hash + Clone + CheckNull,
                  TOA: 'static + Integer + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {
            make_count_by::<MO, TIA, TOA>(size).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TIA, @hashable),
            (TOA, @integers)
        ], (size))
    }
    let size = size as usize;
    let MO = try_!(Type::try_from(MO));
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [(QO, @floats)], (size, MO, TIA, TOA))
}
