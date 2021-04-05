use std::{fmt, ptr};
use std::ffi::{c_void, CStr};
use std::fmt::{Debug, Formatter};
use std::mem::{ManuallyDrop, transmute};
use std::os::raw::c_char;

use opendp::{err};
use opendp::core;
use opendp::core::{ChainMT, ChainTT, Domain, Measure, MeasureGlue, Measurement, Metric, MetricGlue, Transformation};
use opendp::error::*;

use crate::util;
use crate::util::{c_bool, Type};

#[derive(PartialEq)]
pub enum FfiOwnership {
    LIBRARY,
    #[allow(dead_code)]
    CLIENT,
}

pub struct FfiObject {
    pub type_: Type,
    pub value: ManuallyDrop<Box<()>>,
    pub ownership: FfiOwnership,
}

impl FfiObject {
    pub fn new(type_: Type, value: Box<()>, ownership: FfiOwnership) -> Self {
        FfiObject { type_, value: ManuallyDrop::new(value), ownership }
    }

    #[cfg(test)]
    pub fn new_raw_from_type<T: 'static>(value: T) -> *mut Self {
        let type_ = Type::of::<T>();
        let value = util::into_box(value);
        util::into_raw(Self::new(type_, value, FfiOwnership::LIBRARY))
    }

    pub fn as_ref<T>(&self) -> &T {
        // TODO: Check type.
        let value = self.value.as_ref() as *const () as *const T;
        let value = unsafe { value.as_ref() };
        value.unwrap()
    }
}

impl Drop for FfiObject {
    fn drop(&mut self) {
        if &FfiOwnership::LIBRARY == &self.ownership {
            unsafe { ManuallyDrop::drop(&mut self.value) };
        }
    }
}

#[repr(C)]
pub struct FfiSlice {
    pub ptr: *const c_void,
    pub len: usize,
}

impl FfiSlice {
    pub fn new_raw(ptr: *mut c_void, len: usize) -> *mut Self {
        util::into_raw(FfiSlice { ptr, len })
    }
}

#[repr(C)]
pub struct FfiError {
    pub variant: *mut c_char,
    pub message: *mut c_char, // MAY BE NULL!
    pub backtrace: *mut c_char,
}

impl FfiError {
    pub fn new_raw(error: Error) -> *mut Self {
        util::into_raw(Self::from(error))
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

impl From<Error> for FfiError {
    fn from(error: Error) -> Self {
        Self {
            variant: try_!(util::into_c_char_p(format!("{:?}", error.variant))),
            message: try_!(error.message.map_or(Ok(ptr::null::<c_char>() as *mut c_char), util::into_c_char_p)),
            backtrace: try_!(util::into_c_char_p(format!("{:?}", error.backtrace)))
        }
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
    pub fn new(result: Fallible<T>) -> Self {
        result.map_or_else(|e| Self::Err(FfiError::new_raw(e)), Self::Ok)
    }
}

impl<TI, TO: From<TI>> From<Fallible<TI>> for FfiResult<*mut TO> {
    fn from(result: Fallible<TI>) -> Self {
        result.map_or_else(
            |e| Self::Err(util::into_raw(FfiError::from(e))),
            |v| Self::Ok(util::into_raw(TO::from(v))))
    }
}

impl<T> From<Error> for FfiResult<T> {
    fn from(e: Error) -> Self {
        Self::Err(FfiError::new_raw(e))
    }
}

fn new_domain_types<D: 'static + Domain>() -> (Type, Type) {
    let domain_type = Type::of::<D>();
    let domain_carrier = Type::of::<D::Carrier>();
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
    pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Measure>(input_glue: FfiMetricGlue<FfiDomain, FfiMetric>, output_glue: FfiMeasureGlue<FfiDomain, FfiMeasure>, value: Measurement<ID, OD, IM, OM>) -> Self {
        let value = util::into_box(value);
        FfiMeasurement { input_glue, output_glue, value }
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Measure> From<Measurement<DI, DO, MI, MO>> for FfiMeasurement {
    fn from(value: Measurement<DI, DO, MI, MO>) -> Self {
        let input_glue = FfiMetricGlue::<DI, MI>::new();
        let input_glue = unsafe { transmute(input_glue) };
        let output_glue = FfiMeasureGlue::<DO, MO>::new();
        let output_glue = unsafe { transmute(output_glue) };
        FfiMeasurement::new(input_glue, output_glue, value)
    }
}

pub struct FfiTransformation {
    pub input_glue: FfiMetricGlue<FfiDomain, FfiMetric>,
    pub output_glue: FfiMetricGlue<FfiDomain, FfiMetric>,
    pub value: Box<Transformation<FfiDomain, FfiDomain, FfiMetric, FfiMetric>>,
}

impl FfiTransformation {
    pub fn new<ID: 'static + Domain, OD: 'static + Domain, IM: Metric, OM: Metric>(input_glue: FfiMetricGlue<FfiDomain, FfiMetric>, output_glue: FfiMetricGlue<FfiDomain, FfiMetric>, value: Transformation<ID, OD, IM, OM>) -> Self {
        let value = util::into_box(value);
        FfiTransformation { input_glue, output_glue, value }
    }
}

impl<DI: 'static + Domain, DO: 'static + Domain, MI: 'static + Metric, MO: 'static + Metric> From<Transformation<DI, DO, MI, MO>> for FfiTransformation {
    fn from(value: Transformation<DI, DO, MI, MO>) -> Self {
        let input_glue = FfiMetricGlue::<DI, MI>::new();
        let input_glue = unsafe { transmute(input_glue) };
        let output_glue = FfiMetricGlue::<DO, MO>::new();
        let output_glue = unsafe { transmute(output_glue) };
        FfiTransformation::new(input_glue, output_glue, value)
    }
}

#[no_mangle]
pub extern "C" fn opendp_core__error_free(this: *mut FfiError) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_check(this: *const FfiMeasurement, distance_in: *const FfiObject, distance_out: *const FfiObject) -> FfiResult<*mut c_bool> {
    let this = util::as_ref(this);
    let distance_in = util::as_ref(distance_in);
    let distance_out = util::as_ref(distance_out);
    let res = this.value.privacy_relation.eval(&distance_in.value, &distance_out.value);

    FfiResult::new(res.map(util::from_bool).map(util::into_raw))
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> FfiResult<*mut FfiObject> {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    if arg.type_ != this.input_glue.domain_carrier {
        return err!(DomainMismatch, "arg type does not match input domain").into()
    }
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);

    res.map(|o| FfiObject::new(res_type, o, FfiOwnership::LIBRARY)).into()
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
        return err!(DomainMismatch, "arg type does not match input domain").into()
    }
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    res.map(|o| FfiObject::new(res_type, o, FfiOwnership::LIBRARY)).into()
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_mt(measurement1: *const FfiMeasurement, transformation0: *const FfiTransformation) -> FfiResult<*mut FfiMeasurement> {
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
        return err!(DomainMismatch, "chained domain types do not match").into()
    }

    let measurement = try_!(ChainMT::make_chain_mt_glue(
        value1, value0, None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.measure_glue));

    FfiResult::Ok(util::into_raw(
        FfiMeasurement::new(input_glue0.clone(), output_glue1.clone(), measurement)))
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_tt(transformation1: *const FfiTransformation, transformation0: *const FfiTransformation) -> FfiResult<*mut FfiTransformation> {
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
        return err!(DomainMismatch, "chained domain types do not match").into()
    }

    let transformation = ChainTT::make_chain_tt_glue(
        value1,
        value0,
        None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.metric_glue);

    match transformation {
        Ok(trans) => FfiResult::Ok(util::into_raw(FfiTransformation::new(input_glue0.clone(), output_glue1.clone(), trans))),
        Err(e) => e.into()
    }
}

#[no_mangle]
pub extern "C" fn opendp_core__make_composition(measurement0: *const FfiMeasurement, measurement1: *const FfiMeasurement) -> FfiResult<*mut FfiMeasurement> {
    // TODO: This could stand to be restructured the way make_chain_xx was, but there's other cleanup needed here, can do it then.
    let measurement0 = util::as_ref(measurement0);
    let measurement1 = util::as_ref(measurement1);
    if measurement0.input_glue.domain_type != measurement1.input_glue.domain_type {
        return err!(DomainMismatch, "chained domain types do not match").into()
    }
    let input_glue = measurement0.input_glue.clone();
    let output_glue0 = measurement0.output_glue.clone();
    let output_glue1 = measurement1.output_glue.clone();
    // TODO: output_glue for composition.
    let output_glue_domain_type = Type::of::<FfiDomain>();
    let output_glue_domain_carrier = Type::new_box_pair(&output_glue0.domain_carrier, &output_glue1.domain_carrier);
    let output_glue_measure_glue = output_glue0.measure_glue.clone();
    let output_glue = FfiMeasureGlue::<FfiDomain, FfiMeasure>::new_explicit(output_glue_domain_type, output_glue_domain_carrier, output_glue_measure_glue);
    let measurement = try_!(core::make_composition_glue(
        &measurement0.value,
        &measurement1.value,
        &input_glue.metric_glue,
        &output_glue0.measure_glue,
        &output_glue1.measure_glue
    ));

    FfiResult::Ok(util::into_raw(FfiMeasurement::new(input_glue, output_glue, measurement)))
}

#[no_mangle]
pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "error_free", "args": [ ["const FfiError *", "this"] ] },
    { "name": "measurement_check", "args": [ ["const FfiMeasurement *", "this"], ["const FfiObject *", "d_in"], ["const FfiObject *", "d_out"] ], "ret": "FfiResult<bool *>" },
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
                    variant: util::into_c_char_p("FailedFunction".to_owned()).unwrap_assert(),
                    message: util::into_c_char_p("Eat my shorts!".to_owned()).unwrap_assert(),
                    backtrace: util::into_c_char_p("".to_owned()).unwrap_assert(),
                },
                util::into_owned(err)
            )
        }
    }

}
