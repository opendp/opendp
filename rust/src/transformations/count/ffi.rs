use std::convert::TryFrom;
use std::os::raw::c_char;

use crate::core::{FfiResult, IntoAnyTransformationFfiResultExt};
use crate::core::{Metric, MetricSpace};
use crate::domains::{AtomDomain, VectorDomain};
use crate::err;
use crate::error::Fallible;
use crate::ffi::any::{AnyDomain, AnyMetric, Downcast};
use crate::ffi::any::{AnyObject, AnyTransformation};
use crate::ffi::util::{Type, c_bool, to_bool};
use crate::metrics::{L1Distance, L2Distance, SymmetricDistance};
use crate::traits::{Hashable, Integer, Number, Primitive};
use crate::transformations::{
    CountByCategoriesConstant, make_count, make_count_by, make_count_by_categories,
    make_count_distinct,
};

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_count(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        TIA: Primitive,
        TO: Number,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
        make_count::<TIA, TO>(input_domain, input_metric).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA = try_!(input_domain.type_.get_atom());
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @primitives),
        (TO, @numbers)
    ], (input_domain, input_metric))
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_count_distinct(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TIA, TO: 'static>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        TIA: Hashable,
        TO: Number,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TIA>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
        make_count_distinct::<TIA, TO>(input_domain, input_metric).into_any()
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TIA = try_!(input_domain.type_.get_atom());
    let TO = try_!(Type::try_from(TO));
    dispatch!(monomorphize, [
        (TIA, @hashable),
        (TO, @numbers)
    ], (input_domain, input_metric))
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_count_by_categories(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    categories: *const AnyObject,
    null_category: c_bool,
    MO: *const c_char,
    TO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<QO>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
        categories: *const AnyObject,
        null_category: bool,
        MO: Type,
        TI: Type,
        TO: Type,
    ) -> Fallible<AnyTransformation>
    where
        QO: Number,
    {
        fn monomorphize2<MO, TI, TO>(
            input_domain: &AnyDomain,
            input_metric: &AnyMetric,
            categories: *const AnyObject,
            null_category: bool,
        ) -> Fallible<AnyTransformation>
        where
            MO: 'static + Metric + CountByCategoriesConstant<MO::Distance> + Default,
            MO::Distance: Number,
            TI: Hashable,
            TO: Number,
            (VectorDomain<AtomDomain<TO>>, MO): MetricSpace,
        {
            let input_domain = input_domain
                .downcast_ref::<VectorDomain<AtomDomain<TI>>>()?
                .clone();
            let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
            let categories = try_as_ref!(categories).downcast_ref::<Vec<TI>>()?.clone();
            make_count_by_categories::<MO, TI, TO>(
                input_domain,
                input_metric,
                categories,
                null_category,
            )
            .into_any()
        }
        dispatch!(monomorphize2, [
            (MO, [L1Distance<QO>, L2Distance<QO>]),
            (TI, @hashable),
            (TO, @numbers)
        ], (input_domain, input_metric, categories, null_category))
    }
    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let null_category = to_bool(null_category);
    let MO = try_!(Type::try_from(MO));
    let TI = try_!(input_domain.type_.get_atom());
    let TO = try_!(Type::try_from(TO));
    let QO = try_!(MO.get_atom());
    dispatch!(monomorphize, [
        (QO, @numbers)
    ], (input_domain, input_metric, categories, null_category, MO, TI, TO))
    .into()
}

#[unsafe(no_mangle)]
pub extern "C" fn opendp_transformations__make_count_by(
    input_domain: *const AnyDomain,
    input_metric: *const AnyMetric,
    TV: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<TK, TV>(
        input_domain: &AnyDomain,
        input_metric: &AnyMetric,
    ) -> Fallible<AnyTransformation>
    where
        TK: Hashable,
        TV: Integer,
    {
        let input_domain = input_domain
            .downcast_ref::<VectorDomain<AtomDomain<TK>>>()?
            .clone();
        let input_metric = input_metric.downcast_ref::<SymmetricDistance>()?.clone();
        make_count_by::<TK, TV>(input_domain, input_metric).into_any()
    }

    let input_domain = try_as_ref!(input_domain);
    let input_metric = try_as_ref!(input_metric);
    let TK = try_!(input_domain.type_.get_atom());
    let TV = try_!(Type::try_from(TV));

    dispatch!(monomorphize, [
        (TK, @hashable),
        (TV, @integers)
    ], (input_domain, input_metric))
    .into()
}
