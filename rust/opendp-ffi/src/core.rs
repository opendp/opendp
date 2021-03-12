use std::mem::transmute;
use std::os::raw::{c_char, c_uint};

use opendp::core;
use opendp::core::{ChainMT, ChainTT, Domain, Measure, MeasureGlue, Measurement, Metric, MetricGlue, Transformation};

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

    // pub fn into_owned<T>(self) -> T {
    //     // TODO: Check T against self.type_.
    //     let value = Box::into_raw(self.value) as *mut T;
    //     ffi_utils::into_owned(value)
    // }

    pub fn as_ref<T>(&self) -> &T {
        // TODO: Check type.
        let value = self.value.as_ref() as *const () as *const T;
        let value = unsafe { value.as_ref() };
        value.unwrap()
    }
}

pub struct FfiError {
    pub tag: c_uint,
    pub message: *const c_char,
}

impl FfiError {
    pub fn new(error: OdpError) -> *mut Self {
        let tag = match error {
            OdpError::Foo(_) => 1,
            OdpError::Bar(_) => 2,
        };
        let message = util::into_c_char_p(format!("{:?}", error));
        let ffi_error = FfiError { tag, message };
        util::into_raw(ffi_error)
    }
}

pub enum FfiResult {
    Ok(*mut FfiObject),
    Err(*mut FfiError)
}

#[derive(Debug)]
pub enum OdpError {
    Foo(String),
    Bar(String),
}

impl FfiResult {
    pub fn new_typed(type_: Type, result: Result<Box<()>, OdpError>) -> *mut Self {
        let ffi_result = result.map_or_else(|e| Self::Err(FfiError::new(e)), |o| Self::Ok(FfiObject::new_typed(type_, o)));
        util::into_raw(ffi_result)
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
}

#[derive(Clone)]
pub struct FfiMetric;
impl Metric for FfiMetric {
    type Distance = ();
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
pub extern "C" fn opendp_core__measurement_invoke(this: *const FfiMeasurement, arg: *const FfiObject) -> *mut FfiObject {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    assert_eq!(arg.type_, this.input_glue.domain_carrier);
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    // Pretend this returns a result
    let res: Result<_, &'static str> = Ok(res);
    let res = res.unwrap();
    FfiObject::new_typed(res_type, res)
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_invoke_result(this: *const FfiMeasurement, arg: *const FfiObject) -> *mut FfiResult {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    assert_eq!(arg.type_, this.input_glue.domain_carrier);
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    // Pretend this returns a result
    let res: Result<_, OdpError> = Ok(res);
    FfiResult::new_typed(res_type, res)
}

#[no_mangle]
pub extern "C" fn opendp_core__measurement_free(this: *mut FfiMeasurement) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_invoke(this: *const FfiTransformation, arg: *const FfiObject) -> *mut FfiObject {
    let this = util::as_ref(this);
    let arg = util::as_ref(arg);
    assert_eq!(arg.type_, this.input_glue.domain_carrier);
    let res_type = this.output_glue.domain_carrier.clone();
    let res = this.value.function.eval_ffi(&arg.value);
    FfiObject::new_typed(res_type, res)
}

#[no_mangle]
pub extern "C" fn opendp_core__transformation_free(this: *mut FfiTransformation) {
    util::into_owned(this);
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_mt(measurement1: *mut FfiMeasurement, transformation0: *mut FfiTransformation) -> *mut FfiMeasurement {
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

    // TODO: handle returning error variant in *mut FfiMeasurement
    assert_eq!(output_glue0.domain_type, input_glue1.domain_type);

    let measurement = ChainMT::make_chain_mt_glue(
        value1,
        value0,
        None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.measure_glue)
        .expect("mt chain failed. TODO: handle returning error variant");
    FfiMeasurement::new(input_glue0.clone(), output_glue1.clone(), measurement)
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_tt(transformation1: *mut FfiTransformation, transformation0: *mut FfiTransformation) -> *mut FfiTransformation {
    let transformation0 = util::as_ref(transformation0);
    let transformation1 = util::as_ref(transformation1);
    assert_eq!(transformation0.output_glue.domain_type, transformation1.input_glue.domain_type);
    let input_glue = transformation0.input_glue.clone();
    let x_glue = transformation0.output_glue.clone();
    let output_glue = transformation1.output_glue.clone();
    // TODO: Add plumbing for hints from FFI.
    let transformation = ChainTT::make_chain_tt_glue(
        &transformation1.value,
        &transformation0.value,
        None,
        &input_glue.metric_glue,
        &x_glue.metric_glue,
        &output_glue.metric_glue
    ).expect("tt chain failed. TODO: handle returning error variant");
    FfiTransformation::new(input_glue, output_glue, transformation)
}

#[no_mangle]
pub extern "C" fn opendp_core__make_composition(measurement0: *mut FfiMeasurement, measurement1: *mut FfiMeasurement) -> *mut FfiMeasurement {
    let measurement0 = util::as_ref(measurement0);
    let measurement1 = util::as_ref(measurement1);
    assert_eq!(measurement0.input_glue.domain_type, measurement1.input_glue.domain_type);
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
    ).expect("compose failed. TODO: handle returning error variant");
    FfiMeasurement::new(input_glue, output_glue, measurement)
}

#[no_mangle]
pub extern "C" fn opendp_core__bootstrap() -> *const c_char {
    let spec =
r#"{
"functions": [
    { "name": "measurement_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
    { "name": "measurement_free", "args": [ ["void *", "this"] ] },
    { "name": "transformation_invoke", "args": [ ["const void *", "this"], ["void *", "arg"] ], "ret": "void *" },
    { "name": "transformation_free", "args": [ ["void *", "this"] ] },
    { "name": "make_chain_mt", "args": [ ["void *", "measurement"], ["void *", "transformation"] ], "ret": "void *" },
    { "name": "make_chain_tt", "args": [ ["void *", "transformation1"], ["void *", "transformation0"] ], "ret": "void *" },
    { "name": "make_composition", "args": [ ["void *", "transformation0"], ["void *", "transformation1"] ], "ret": "void *" }
]
}"#;
    util::bootstrap(spec)
}
