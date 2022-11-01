use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::{Metric, MetricSpace};
use crate::domains::{AllDomain, MapDomain, VectorDomain};
use crate::err;
use crate::ffi::any::Downcast;
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::util::{c_bool, to_bool, Type};
use crate::metrics::{L1Distance, L2Distance};
use crate::traits::{Float, Hashable, Number, Primitive};
use crate::transformations::{
    make_count, make_count_by, make_count_by_categories, make_count_distinct,
    CountByCategoriesConstant, CountByConstant,
};

#[no_mangle]
pub extern "C" fn opendp_transformations__make_count(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO>() -> FfiResult<*mut AnyTransformation>
    where
        TIA: Primitive,
        TO: Number,
    {
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
pub extern "C" fn opendp_transformations__make_count_distinct(
    TIA: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO: 'static>() -> FfiResult<*mut AnyTransformation>
    where
        TIA: Hashable,
        TO: Number,
    {
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
pub extern "C" fn opendp_transformations__make_count_by_categories(
    categories: *const AnyObject,
    null_category: c_bool,
    MO: *const c_char,
    TI: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        categories: *const AnyObject,
        null_category: bool,
        MO: Type,
        TI: Type,
        TO: Type,
    ) -> FfiResult<*mut AnyTransformation>
    where
        QO: Number,
    {
        fn monomorphize2<MO, TI, TO>(
            categories: *const AnyObject,
            null_category: bool,
        ) -> FfiResult<*mut AnyTransformation>
        where
            MO: 'static + Metric + CountByCategoriesConstant<MO::Distance>,
            MO::Distance: Number,
            TI: Hashable,
            TO: Number,
            (VectorDomain<AllDomain<TO>>, MO): MetricSpace,
        {
            let categories = try_!(try_as_ref!(categories).downcast_ref::<Vec<TI>>()).clone();
            make_count_by_categories::<MO, TI, TO>(categories, null_category).into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TI, @hashable),
            (TO, @numbers)
        ], (categories, null_category))
    }
    let null_category = to_bool(null_category);
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(Type::try_from(TI));
    let TO = try_!(Type::try_from(TO));
    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [
        (QO, @numbers)
    ], (categories, null_category, MO, TI, TO))
}

#[no_mangle]
pub extern "C" fn opendp_transformations__make_count_by(
    MO: *const c_char,
    TK: *const c_char,
    TV: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(MO: Type, TK: Type, TV: Type) -> FfiResult<*mut AnyTransformation>
    where
        QO: Float,
    {
        fn monomorphize2<MO, TK, TV>() -> FfiResult<*mut AnyTransformation>
        where
            MO: 'static + Metric + CountByConstant<MO::Distance>,
            MO::Distance: Float,
            TK: Hashable,
            TV: Number,
            (MapDomain<AllDomain<TK>, AllDomain<TV>>, MO): MetricSpace,
        {
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
