use std::{fmt, ptr};
use std::ffi::CStr;
use std::fmt::{Debug, Formatter};
use std::mem::transmute;
use std::os::raw::c_char;

use opendp::{err, fallible};
use opendp::core;
use opendp::core::{ChainMT, ChainTT, Domain, Measure, MeasureGlue, Measurement, Metric, MetricGlue, Transformation};
use opendp::error::Error;

use crate::util;
use crate::util::Type;

pub struct FfiObject {
    pub type_: Type,
    pub value: Box<()>,
}

impl FfiObject {
    pub fn new_typed(type_: Type, value: Box<()>) -> *mut FfiObject {
        let object = FfiObject { type_, value };
        util::into_raw(object)
    }

    pub fn new<T: 'static>(value: T) -> *mut FfiObject {
        let type_ = Type::new::<T>();
        let value = util::into_box(value);
        Self::new_typed(type_, value)
    }

    pub fn as_ref<T>(&self) -> &T {
        // TODO: Check type.
        let value = self.value.as_ref() as *const () as *const T;
        let value = unsafe { value.as_ref() };
        value.unwrap()
    }
}

#[repr(C)]
pub struct FfiError {
    pub variant: *mut c_char,
    pub message: *mut c_char, // MAY BE NULL!
    pub backtrace: *mut c_char,
}

impl FfiError {
    pub fn new(error: Error) -> *mut Self {
        let ffi_error = FfiError {
            variant: util::into_c_char_p(format!("{:?}", error.variant)),
            message: error.message.map_or(ptr::null::<c_char>() as *mut c_char, util::into_c_char_p),
            backtrace: util::into_c_char_p(format!("{:?}", error.backtrace))
        };
        util::into_raw(ffi_error)
    }

    fn variant_str(&self) -> &str {
        unsafe { CStr::from_ptr(self.variant).to_str().unwrap_or("Couldn't get variant!") }
    }

    fn message_str(&self) -> Option<&str> {
        unsafe { self.message.as_ref().map(|s| CStr::from_ptr(s).to_str().unwrap_or("Couldn't get message!")) }
    }
}

impl Drop for FfiError {
    fn drop(&mut self) {
        let _variant = util::into_string(self.variant);
        let _message = unsafe { self.message.as_mut() }.map(|p| util::into_string(p));
        let _backtrace = util::into_string(self.backtrace);
    }
}

// Handy stuff for tests and debugging.
impl PartialEq for FfiError {
    fn eq(&self, other: &Self) -> bool {
        self.variant_str() == other.variant_str() && self.message_str() == other.message_str()
    }
}

impl Debug for FfiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "FfiError: {{ type: {}, message: {:?} }}", self.variant_str(), self.message_str())
    }
}

// Using this repr means we'll get a tagged union in C.
// Because this is a generic, we need to be careful about sizes. Currently, everything that goes in here
// is a pointer, so we're OK, but we may need to revisit this.
#[repr(C, u32)]
pub enum FfiResult<T> {
    Ok(T),
    Err(*mut FfiError)
}

impl<T> FfiResult<T> {
    pub fn new(result: Result<T, Error>) -> Self {
        result.map_or_else(|e| Self::Err(FfiError::new(e)), |o| Self::Ok(o))
    }
}

fn new_domain_types<D: 'static + Domain>() -> (Type, Type) {
    let domain_type = Type::new::<D>();
    let domain_carrier = Type::new::<D::Carrier>();
    (domain_type, domain_carrier)
}

#[derive(Clone)]
pub struct FfiMetricGlue<D: Domain, M: Metric> {
    pub domain_type: Type,
    pub domain_carrier: Type,
    pub metric_glue: MetricGlue<D, M>,
}
impl<D: 'static + Domain, M: 'static + Metric> FfiMetricGlue<D, M> {
    pub fn new() -> Self {
        let (domain_type, domain_carrier) = new_domain_types::<D>();
        let metric_glue = MetricGlue::new();
        Self::new_explicit(domain_type, domain_carrier, metric_glue)
    }

    pub fn new_explicit(domain_type: Type, domain_carrier: Type, metric_glue: MetricGlue<D, M>) -> Self {
        FfiMetricGlue { domain_type, domain_carrier, metric_glue }
    }
}

#[derive(Clone)]
pub struct FfiMeasureGlue<D: Domain, M: Measure> {
    pub domain_type: Type,
    pub domain_carrier: Type,
    pub measure_glue: MeasureGlue<D, M>,
}
impl<D: 'static + Domain, M: 'static + Measure> FfiMeasureGlue<D, M> {
    pub fn new() -> Self {
        let (domain_type, domain_carrier) = new_domain_types::<D>();
        let measure_glue = MeasureGlue::new();
        Self::new_explicit(domain_type, domain_carrier, measure_glue)
    }
    pub fn new_explicit(domain_type: Type, domain_carrier: Type, measure_glue: MeasureGlue<D, M>) -> Self {
        FfiMeasureGlue { domain_type, domain_carrier, measure_glue }
    }
}

#[derive(Clone, PartialEq)]
pub struct FfiDomain;
impl Domain for FfiDomain {
    type Carrier = ();
    fn member(&self, _val: &Self::Carrier) -> bool { unimplemented!() }
}

#[derive(Clone)]
pub struct FfiMeasure;
impl Measure for FfiMeasure {
    type Distance = ();

    fn new() -> Self { unreachable!() }
}

#[derive(Clone)]
pub struct FfiMetric;
impl Metric for FfiMetric {
    type Distance = ();
    fn new() -> Self { unreachable!() }
}

pub struct FfiMeasurement {
    pub input_glue: FfiMetricGlue<FfiDomain, FfiMetric>,
    pub output_glue: FfiMeasureGlue<FfiDomain, FfiMeasure>,
    pub value: Box<Measurement<FfiDomain, FfiDomain, FfiMetric, FfiMeasure>>,
}

impl FfiMeasurement {
    pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, OM: 'static + Measure>(value: Measurement<ID, OD, IM, OM>) -> *mut FfiMeasurement {
        let input_glue = FfiMetricGlue::<ID, IM>::new();
        let input_glue = unsafe { transmute(input_glue) };
        let output_glue = FfiMeasureGlue::<OD, OM>::new();
        let output_glue = unsafe { transmute(output_glue) };
        Self::new(input_glue, output_glue, value)
    }

    pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Measure>(input_glue: FfiMetricGlue<FfiDomain, FfiMetric>, output_glue: FfiMeasureGlue<FfiDomain, FfiMeasure>, value: Measurement<ID, OD, IM, OM>) -> *mut FfiMeasurement {
        let value = util::into_box(value);
        let ffi_measurement = FfiMeasurement { input_glue, output_glue, value };
        util::into_raw(ffi_measurement)
    }
}

pub struct FfiTransformation {
    pub input_glue: FfiMetricGlue<FfiDomain, FfiMetric>,
    pub output_glue: FfiMetricGlue<FfiDomain, FfiMetric>,
    pub value: Box<Transformation<FfiDomain, FfiDomain, FfiMetric, FfiMetric>>,
}

impl FfiTransformation {
    pub fn new_from_types<ID: 'static + Domain, OD: 'static + Domain, IM: 'static + Metric, OM: 'static + Metric>(value: Transformation<ID, OD, IM, OM>) -> *mut FfiTransformation {
        let input_glue = FfiMetricGlue::<ID, IM>::new();
        let input_glue = unsafe { transmute(input_glue) };
        let output_glue = FfiMetricGlue::<OD, OM>::new();
        let output_glue = unsafe { transmute(output_glue) };
        Self::new(input_glue, output_glue, value)
    }

    pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Metric>(input_glue: FfiMetricGlue<FfiDomain, FfiMetric>, output_glue: FfiMetricGlue<FfiDomain, FfiMetric>, value: Transformation<ID, OD, IM, OM>) -> *mut FfiTransformation {
        let value = util::into_box(value);
        let ffi_transformation = FfiTransformation { input_glue, output_glue, value };
        util::into_raw(ffi_transformation)
    }
}

#[no_mangle]
pub extern "C" fn opendp_core__error_free(this: *mut FfiError) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> FfiResult<*mut FfiObject> {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    if arg.type_ != this.input_glue.domain_carrier {
        return FfiResult::new(fallible!(DomainMismatch))
    }
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    let res = res.map(|o| FfiObject::new_typed(res_type, o));
    FfiResult::new(res)
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_free(this: *mut FfiMeasurement) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_invoke(this: *const FfiTransformation, arg: *const FfiObject) -> FfiResult<*mut FfiObject> {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    if arg.type_ != this.input_glue.domain_carrier {
        return FfiResult::new(fallible!(DomainMismatch))
    }
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    let res = res.map(|o| FfiObject::new_typed(res_type, o));
    FfiResult::new(res)
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_mt(measurement1: *mut FfiMeasurement, transformation0: *mut FfiTransformation) -> FfiResult<*mut FfiMeasurement> {
    let transformation0 = util::as_ref(transformation0);
    let measurement1 = util::as_ref(measurement1);

    let FfiTransformation {
        input_glue: input_glue0,
        output_glue: output_glue0,
        value: value0
    } = transformation0;

    let FfiMeasurement {
        input_glue: input_glue1,
        output_glue: output_glue1,
        value: value1
    } = measurement1;

    if output_glue0.domain_type != input_glue1.domain_type {
        return FfiResult::new(fallible!(DomainMismatch))
    }

    let measurement = ChainMT::make_chain_mt_glue(
        value1,
        value0,
        None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.measure_glue);
    let measurement = measurement.map(|o| FfiMeasurement::new(input_glue0.clone(), output_glue1.clone(), o));
    FfiResult::new(measurement)
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_tt(transformation1: *mut FfiTransformation, transformation0: *mut FfiTransformation) -> FfiResult<*mut FfiTransformation> {
    let transformation0 = util::as_ref(transformation0);
    let transformation1 = util::as_ref(transformation1);

    let FfiTransformation {
        input_glue: input_glue0,
        output_glue: output_glue0,
        value: value0
    } = transformation0;

    let FfiTransformation {
        input_glue: input_glue1,
        output_glue: output_glue1,
        value: value1
    } = transformation1;

    if output_glue0.domain_type != input_glue1.domain_type {
        return FfiResult::new(fallible!(DomainMismatch))
    }

    let transformation = ChainTT::make_chain_tt_glue(
        value1,
        value0,
        None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.metric_glue);
    let transformation = transformation.map(|o| FfiTransformation::new(input_glue0.clone(), output_glue1.clone(), o));
    FfiResult::new(transformation)
}

#[no_mangle]
pub extern "C" fn opendp_core__make_composition(measurement0: *mut FfiMeasurement, measurement1: *mut FfiMeasurement) -> FfiResult<*mut FfiMeasurement> {
    // TODO: This could stand to be restructured the way make_chain_xx was, but there's other cleanup needed here, can do it then.
    let measurement0 = util::as_ref(measurement0);
    let measurement1 = util::as_ref(measurement1);
    if measurement0.input_glue.domain_type != measurement1.input_glue.domain_type {
        return FfiResult::new(fallible!(DomainMismatch))
    }
    let input_glue = measurement0.input_glue.clone();
    let output_glue0 = measurement0.output_glue.clone();
    let output_glue1 = measurement1.output_glue.clone();
    // TODO: output_glue for composition.
    let output_glue_domain_type = Type::new::<FfiDomain>();
    let output_glue_domain_carrier = Type::new_box_pair(&output_glue0.domain_carrier, &output_glue1.domain_carrier);
    let output_glue_measure_glue = output_glue0.measure_glue.clone();
    let output_glue = FfiMeasureGlue::<FfiDomain, FfiMeasure>::new_explicit(output_glue_domain_type, output_glue_domain_carrier, output_glue_measure_glue);
    let measurement = core::make_composition_glue(
        &measurement0.value,
        &measurement1.value,
        &input_glue.metric_glue,
        &output_glue0.measure_glue,
        &output_glue1.measure_glue
    );
    let measurement = measurement.map(|o| FfiMeasurement::new(input_glue, output_glue, o));
    FfiResult::new(measurement)
}

#[no_mangle]
pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "error_free", "args": [ ["const FfiError *", "this"] ] },
    { "name": "measurement_invoke", "args": [ ["const FfiMeasurement *", "this"], ["const FfiObject *", "arg"] ], "ret": "FfiResult<FfiObject *>" },
    { "name": "measurement_free", "args": [ ["FfiMeasurement *", "this"] ] },
    { "name": "transformation_invoke", "args": [ ["const FfiTransformation *", "this"], ["const FfiObject *", "arg"] ], "ret": "FfiResult<FfiObject *>" },
    { "name": "transformation_free", "args": [ ["FfiTransformation *", "this"] ] },
    { "name": "make_chain_mt", "args": [ ["const FfiMeasurement *", "measurement"], ["const FfiTransformation *", "transformation"] ], "ret": "FfiResult<FfiMeasurement *>" },
    { "name": "make_chain_tt", "args": [ ["const FfiTransformation *", "transformation1"], ["const FfiTransformation *", "transformation0"] ], "ret": "FfiResult<FfiTransformation *>" },
    { "name": "make_composition", "args": [ ["const FfiMeasurement *", "transformation0"], ["const FfiMeasurement *", "transformation1"] ], "ret": "FfiResult<FfiMeasurement *>" }
]
}"#;
    util::bootstrap(spec)
}

// UNIT TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ffi_result_ok() {
        let res = 999;
        let res = Ok(res);
        let ffi_res = FfiResult::new(res);
        match ffi_res {
            FfiResult::Ok(ok) => assert_eq!(999, ok),
            FfiResult::Err(_) => assert!(false, "Got Err!"),
        }
    }

    #[test]
    fn test_ffi_result_err() {
        let res: Result<(), _> = fallible!(FailedFunction, "Eat my shorts!");
        let ffi_res = FfiResult::new(res);
        match ffi_res {
            FfiResult::Ok(_) => assert!(false, "Got Ok!"),
            FfiResult::Err(err) => assert_eq!(
                FfiError {
                    variant: util::into_c_char_p("FailedFunction".to_owned()),
                    message: util::into_c_char_p("Eat my shorts!".to_owned()),
                    backtrace: util::into_c_char_p("".to_owned()),
                },
                util::into_owned(err)
            )
        }
    }

}
