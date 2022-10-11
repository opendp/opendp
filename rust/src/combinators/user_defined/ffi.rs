use std::ffi::c_char;

use opendp_derive::bootstrap;

use crate::{
    core::{Domain, FfiResult, Function, Metric, StabilityMap, Transformation, Measure, PrivacyMap, Measurement},
    domains::{AllDomain, VectorDomain},
    error::Fallible,
    ffi::{
        any::{AnyDomain, AnyMetric, AnyObject, AnyTransformation, AnyMeasurement, AnyMeasure, IntoAnyStabilityMapExt},
        util::{self, Type},
    },
    metrics::{SymmetricDistance, AgnosticMetric, AbsoluteDistance}, 
    measures::MaxDivergence,
};

type CallbackFn = extern "C" fn(*const AnyObject) -> *mut FfiResult<*mut AnyObject>;

#[bootstrap(
    name = "make_custom_transformation_with_defaults",
    features("contrib", "honest-but-curious"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        stability_map(rust_type = "$metric_distance_type(MO)"),
        DI(c_type = "char *", rust_type = b"null"),
        DO(c_type = "char *", rust_type = b"null"),
        MI(c_type = "char *", rust_type = b"null"),
        MO(c_type = "char *", rust_type = b"null"),
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
    fn monomorphize<DI, DO, MI, MO>(
        function: CallbackFn,
        stability_map: CallbackFn,
    ) -> FfiResult<*mut AnyTransformation>
    where
        DI: 'static + Domain + Default,
        DO: 'static + Domain + Default,
        MI: 'static + Metric + Default,
        MO: 'static + Metric + Default,
    {
        fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
            move |arg: &AnyObject| -> Fallible<AnyObject> {
                util::into_owned(func(arg as *const AnyObject))?.into()
            }
        }
        FfiResult::Ok(util::into_raw(Transformation::new(
            AnyDomain::new(DI::default()),
            AnyDomain::new(DO::default()),
            Function::new_fallible(wrap_func(function)),
            AnyMetric::new(MI::default()),
            AnyMetric::new(MO::default()),
            StabilityMap::new_fallible(wrap_func(stability_map)),
        )))
    }
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));

    dispatch!(monomorphize, [
        (DI, [AllDomain<i32>, VectorDomain<AllDomain<i32>>]),
        (DO, [AllDomain<i32>, VectorDomain<AllDomain<i32>>]),
        (MI, [SymmetricDistance]),
        (MO, [SymmetricDistance])
    ], (function, stability_map))
}



#[bootstrap(
    name = "make_custom_measurement_with_defaults",
    features("contrib", "honest-but-curious"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        privacy_map(rust_type = "$measure_distance_type(MO)"),
        DI(c_type = "char *", rust_type = b"null"),
        DO(c_type = "char *", rust_type = b"null"),
        MI(c_type = "char *", rust_type = b"null"),
        MO(c_type = "char *", rust_type = b"null"),
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
    fn monomorphize<DI, DO, MI, MO>(
        function: CallbackFn,
        privacy_map: CallbackFn,
    ) -> FfiResult<*mut AnyMeasurement>
    where
        DI: 'static + Domain + Default,
        DO: 'static + Domain + Default,
        MI: 'static + Metric + Default,
        MO: 'static + Measure + Default,
    {
        fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
            move |arg: &AnyObject| -> Fallible<AnyObject> {
                util::into_owned(func(arg as *const AnyObject))?.into()
            }
        }
        FfiResult::Ok(util::into_raw(Measurement::new(
            AnyDomain::new(DI::default()),
            AnyDomain::new(DO::default()),
            Function::new_fallible(wrap_func(function)),
            AnyMetric::new(MI::default()),
            AnyMeasure::new(MO::default()),
            PrivacyMap::new_fallible(wrap_func(privacy_map)),
        )))
    }
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));
    let MI = try_!(Type::try_from(MI));
    let MO = try_!(Type::try_from(MO));

    dispatch!(monomorphize, [
        (DI, [AllDomain<i32>, VectorDomain<AllDomain<i32>>]),
        (DO, [AllDomain<i32>, VectorDomain<AllDomain<i32>>]),
        (MI, [AbsoluteDistance<i32>]),
        (MO, [MaxDivergence<f64>])
    ], (function, privacy_map))
}



#[bootstrap(
    name = "make_custom_postprocessor_with_defaults",
    features("contrib"),
    arguments(
        function(rust_type = "$domain_carrier_type(DO)"),
        DI(c_type = "char *", rust_type = b"null"),
        DO(c_type = "char *", rust_type = b"null"),
    ),
    dependencies("c_function")
)]
#[no_mangle]
pub extern "C" fn opendp_combinators__make_custom_postprocessor_with_defaults(
    function: CallbackFn,
    DI: *const c_char,
    DO: *const c_char,
) -> FfiResult<*mut AnyTransformation> {
    fn monomorphize<DI, DO>(
        function: CallbackFn,
    ) -> FfiResult<*mut AnyTransformation>
    where
        DI: 'static + Domain + Default,
        DO: 'static + Domain + Default,
    {
        fn wrap_func(func: CallbackFn) -> impl Fn(&AnyObject) -> Fallible<AnyObject> {
            move |arg: &AnyObject| -> Fallible<AnyObject> {
                util::into_owned(func(arg as *const AnyObject))?.into()
            }
        }
        FfiResult::Ok(util::into_raw( Transformation::new(
            AnyDomain::new(DI::default()),
            AnyDomain::new(DO::default()),
            Function::new_fallible(wrap_func(function)),
            AnyMetric::new(AgnosticMetric::default()),
            AnyMetric::new(AgnosticMetric::default()),
            StabilityMap::<AgnosticMetric, AgnosticMetric>::new(|_| ()).into_any()
        )))
    }
    let DI = try_!(Type::try_from(DI));
    let DO = try_!(Type::try_from(DO));

    dispatch!(monomorphize, [
        (DI, [AllDomain<f64>, VectorDomain<AllDomain<f64>>]),
        (DO, [AllDomain<f64>, VectorDomain<AllDomain<f64>>])
    ], (function))
}
