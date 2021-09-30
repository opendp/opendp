use std::convert::TryFrom;
use std::hash::Hash;
use std::os::raw::{c_char, c_uint};

use num::{Bounded, Integer, One, Zero, Float};
use num::traits::FloatConst;

use opendp::core::{SensitivityMetric};
use opendp::dist::{L1Distance, L2Distance, IntDistance};
use opendp::err;
use opendp::traits::{DistanceConstant, InfCast, ExactIntCast, SaturatingAdd, CheckNull};
use opendp::trans::{CountByConstant, make_count, make_sized_count_by, make_count_by_categories, make_count_distinct, make_sized_count_by_categories, SizedCountByConstant};

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
    MO: *const c_char, TIA: *const c_char
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<QO>(
        categories: *const AnyObject,
        MO: Type, TIA: Type
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant<IntDistance> + One + Integer + Zero + SaturatingAdd + CheckNull,
              IntDistance: InfCast<QO> {

        fn monomorphize2<MO, TIA>(categories: *const AnyObject) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + One + Integer + Zero + SaturatingAdd + CheckNull,
                  TIA: 'static + Eq + Hash + Clone + CheckNull,
                  IntDistance: InfCast<MO::Distance> {

            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TIA >>()).clone();
            make_count_by_categories::<MO, TIA>(categories).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TIA, @hashable)
        ], (categories))
    }
    let MO = try_!(Type::try_from(MO));
    let TIA = try_!(Type::try_from(TIA));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (QO, @integers)
    ], (categories, MO, TIA))
}


#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_count_by_categories(
    size: c_uint, categories: *const AnyObject,
    MO: *const c_char, TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {

    fn monomorphize<QO>(
        size: usize, categories: *const AnyObject,
        MO: Type, TIA: Type, TOA: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant<IntDistance> + One + Float + InfCast<u8>,
              IntDistance: InfCast<QO> {

        fn monomorphize2<MO, TIA, TOA>(
            size: usize, categories: *const AnyObject
        ) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + SizedCountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + One + Float + InfCast<u8>,
                  TIA: 'static + Eq + Hash + Clone + CheckNull,
                  TOA: 'static + Integer + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {

            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TIA >>()).clone();
            make_sized_count_by_categories::<MO, TIA, TOA>(size, categories).into_any()
        }

        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TIA, @hashable),
            (TOA, @integers)
        ], (size, categories))
    }
    let size = size as usize;
    let MO = try_!(Type::try_from(MO));
    let TIA = try_!(Type::try_from(TIA));
    let TOA = try_!(Type::try_from(TOA));

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [
        (QO, @floats)
    ], (size, categories, MO, TIA, TOA))
}

#[no_mangle]
pub extern "C" fn opendp_trans__make_sized_count_by(
    size: c_uint,
    MO: *const c_char, TIA: *const c_char, TOA: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        size: usize, MO: Type, TIA: Type, TOA: Type,
    ) -> FfiResult<*mut AnyTransformation>
        where QO: DistanceConstant<IntDistance> + FloatConst + One,
              IntDistance: InfCast<QO> {
        fn monomorphize2<MO, TIA, TOA>(size: usize) -> FfiResult<*mut AnyTransformation>
            where MO: 'static + SensitivityMetric + CountByConstant<MO::Distance>,
                  MO::Distance: DistanceConstant<IntDistance> + FloatConst + One,
                  TIA: 'static + Eq + Hash + Clone + CheckNull,
                  TOA: 'static + Integer + Zero + One + SaturatingAdd + CheckNull,
                  IntDistance: InfCast<MO::Distance> {
            make_sized_count_by::<MO, TIA, TOA>(size).into_any()
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

    let QO = try_!(MO.get_sensitivity_distance());
    dispatch!(monomorphize, [(QO, @floats)], (size, MO, TIA, TOA))
}
