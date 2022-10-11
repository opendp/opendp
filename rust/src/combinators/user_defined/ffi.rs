use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{
        Domain, FfiResult, Function, Measure, Measurement, Metric, PrivacyMap, StabilityMap,
        Transformation,
    },
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{
            AnyDomain, AnyMeasure, AnyMeasurement, AnyMetric, AnyObject, AnyTransformation,
            IntoAnyStabilityMapExt,
        },
        util::{self, Type, TypeContents},
    },
    measures::{FixedSmoothedMaxDivergence, MaxDivergence, ZeroConcentratedDivergence},
    metrics::{
        AbsoluteDistance, AgnosticMetric, ChangeOneDistance, DiscreteDistance, HammingDistance,
        InsertDeleteDistance, L1Distance, L2Distance, SymmetricDistance,
    },
    traits::CheckNull,
};

type CallbackFn = extern "C" fn(*const AnyObject) -> *mut FfiResult<*mut AnyObject>;

// wrap a CallbackFn in a closure, so that it can be used in transformations and measurements
fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
    move |arg: &AnyObject| -> Fallible<AnyObject> {
        util::into_owned(func(arg as *const AnyObject))?.into()
    }
}

pub(crate) fn default_domain(D: Type) -> Fallible<AnyDomain> {
    let TA = D.get_atom()?;
    fn monomorphize<TA>(D: Type) -> Fallible<AnyDomain>
    where
        TA: 'static + CheckNull,
    {
        fn monomorphize<D>() -> Fallible<AnyDomain>
        where
            D: 'static + Domain + Default,
        {
            Ok(AnyDomain::new(D::default()))
        }
        dispatch!(monomorphize, [(D, [AllDomain<TA>, VectorDomain<AllDomain<TA>>])], ())
    }
    dispatch!(monomorphize, [(TA, @primitives)], (D))
}

pub(crate) fn default_metric(M: Type) -> Fallible<AnyMetric> {
    match &M.contents {
        TypeContents::PLAIN(_) => {
            fn monomorphize<M>() -> Fallible<AnyMetric>
            where
                M: 'static + Metric + Default,
            {
                Ok(AnyMetric::new(M::default()))
            }
            dispatch!(
                monomorphize,
                [(
                    M,
                    [
                        SymmetricDistance,
                        InsertDeleteDistance,
                        ChangeOneDistance,
                        HammingDistance,
                        DiscreteDistance
                    ]
                )],
                ()
            )
        }
        TypeContents::GENERIC { .. } => {
            let QA = M.get_atom()?;
            fn monomorphize<QA>(M: Type) -> Fallible<AnyMetric>
            where
                QA: 'static + CheckNull,
            {
                fn monomorphize<M>() -> Fallible<AnyMetric>
                where
                    M: 'static + Metric + Default,
                {
                    Ok(AnyMetric::new(M::default()))
                }
                dispatch!(monomorphize, [(M, [AbsoluteDistance<QA>, L1Distance<QA>, L2Distance<QA>])], ())
            }
            dispatch!(monomorphize, [(QA, @numbers)], (M))
        }
        _ => fallible!(FFI, "unrecognized metric: {}", M.to_string()),
    }
}
pub(crate) fn default_measure(M: Type) -> Fallible<AnyMeasure> {
    let QA = M.get_atom()?;
    fn monomorphize<QA>(M: Type) -> Fallible<AnyMeasure>
    where
        QA: 'static + CheckNull,
    {
        fn monomorphize<M>() -> Fallible<AnyMeasure>
        where
            M: 'static + Measure + Default,
        {
            Ok(AnyMeasure::new(M::default()))
        }
        dispatch!(monomorphize, [(M, [MaxDivergence<QA>, FixedSmoothedMaxDivergence<QA>, ZeroConcentratedDivergence<QA>])], ())
    }
    dispatch!(monomorphize, [(QA, @numbers)], (M))
}

#[bootstrap(
    name = "make_custom_transformation_with_defaults",
    features("contrib", "honest-but-curious"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        stability_map(rust_type = "$metric_distance_type(MO)"),
        DI(rust_type = b"null"),
        DO(rust_type = b"null"),
        MI(rust_type = b"null"),
        MO(rust_type = b"null"),
    ),
    dependencies("c_function", "c_stability_map")
)]
#[no_mangle]
pub extern "C" fn opendp_combinators__make_custom_transformation_with_defaults(
    function: CallbackFn,
    stability_map: CallbackFn,
    DI: *const c_char,
    DO: *const c_char,
    MI: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));

    FfiResult::Ok(util::into_raw(Transformation::new(
        try_!(default_domain(DI)),
        try_!(default_domain(DO)),
        Function::new_fallible(wrap_func(function)),
        try_!(default_metric(MI)),
        try_!(default_metric(MO)),
        StabilityMap::new_fallible(wrap_func(stability_map)),
    )))
}

#[bootstrap(
    name = "make_custom_measurement_with_defaults",
    features("contrib", "honest-but-curious"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        privacy_map(rust_type = "$measure_distance_type(MO)"),
        DI(rust_type = b"null"),
        DO(rust_type = b"null"),
        MI(rust_type = b"null"),
        MO(rust_type = b"null"),
    ),
    dependencies("c_function", "c_privacy_map")
)]
#[no_mangle]
pub extern "C" fn opendp_combinators__make_custom_measurement_with_defaults(
    function: CallbackFn,
    privacy_map: CallbackFn,
    DI: *const c_char,
    DO: *const c_char,
    MI: *const c_char,
    MO: *const c_char,
) -> FfiResult<*mut AnyMeasurement> {
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));

    FfiResult::Ok(util::into_raw(Measurement::new(
        try_!(default_domain(DI)),
        try_!(default_domain(DO)),
        Function::new_fallible(wrap_func(function)),
        try_!(default_metric(MI)),
        try_!(default_measure(MO)),
        PrivacyMap::new_fallible(wrap_func(privacy_map)),
    )))
}

#[bootstrap(
    name = "make_custom_postprocessor_with_defaults",
    features("contrib"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        DI(rust_type = b"null"),
        DO(rust_type = b"null"),
    ),
    dependencies("c_function")
)]
#[no_mangle]
pub extern "C" fn opendp_combinators__make_custom_postprocessor_with_defaults(
    function: CallbackFn,
    DI: *const c_char,
    DO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));

    FfiResult::Ok(util::into_raw(Transformation::new(
        try_!(default_domain(DI)),
        try_!(default_domain(DO)),
        Function::new_fallible(wrap_func(function)),
        AnyMetric::new(AgnosticMetric::default()),
        AnyMetric::new(AgnosticMetric::default()),
        StabilityMap::<AgnosticMetric, AgnosticMetric>::new(|_| ()).into_any(),
    )))
}
