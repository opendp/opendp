use opendp::err;

use crate::core::{FfiDomain, FfiMeasure, FfiMeasureGlue, FfiMeasurement, FfiResult, FfiTransformation};
use crate::util;
use crate::util::Type;
use opendp::chain::{make_chain_mt_glue, make_chain_tt_glue, make_composition_glue};


#[no_mangle]
pub extern "C" fn opendp_core__make_chain_mt(measurement1: *const FfiMeasurement, transformation0: *const FfiTransformation) -> FfiResult<*mut FfiMeasurement> {
    let transformation0 = try_as_ref!(transformation0);
    let measurement1 = try_as_ref!(measurement1);

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
        return err!(DomainMismatch, "chained domain types do not match").into();
    }

    let measurement = try_!(make_chain_mt_glue(
        value1, value0, None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.measure_glue));

    FfiResult::Ok(util::into_raw(
        FfiMeasurement::new(input_glue0.clone(), output_glue1.clone(), measurement)))
}

#[no_mangle]
pub extern "C" fn opendp_core__make_chain_tt(transformation1: *const FfiTransformation, transformation0: *const FfiTransformation) -> FfiResult<*mut FfiTransformation> {
    let transformation0 = try_as_ref!(transformation0);
    let transformation1 = try_as_ref!(transformation1);

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
        return err!(DomainMismatch, "chained domain types do not match").into();
    }

    let transformation = try_!(make_chain_tt_glue(
        value1,
        value0,
        None,
        &input_glue0.metric_glue,
        &output_glue0.metric_glue,
        &output_glue1.metric_glue));

    FfiResult::Ok(util::into_raw(
        FfiTransformation::new(input_glue0.clone(), output_glue1.clone(), transformation)))
}

#[no_mangle]
pub extern "C" fn opendp_core__make_composition(measurement0: *const FfiMeasurement, measurement1: *const FfiMeasurement) -> FfiResult<*mut FfiMeasurement> {
    let measurement0 = try_as_ref!(measurement0);
    let measurement1 = try_as_ref!(measurement1);

    let FfiMeasurement {
        input_glue: input_glue0,
        output_glue: output_glue0,
        value: value0
    } = measurement0;


    let FfiMeasurement {
        input_glue: input_glue1,
        output_glue: output_glue1,
        value: value1
    } = measurement1;


    if input_glue0.domain_type != input_glue1.domain_type {
        return err!(DomainMismatch, "chained domain types do not match").into();
    }

    let measurement = try_!(make_composition_glue(
        value0, value1,
        &input_glue0.metric_glue,
        &output_glue0.measure_glue,
        &output_glue1.measure_glue));

    // TODO: output_glue for composition.
    let output_glue_domain_type = Type::of::<FfiDomain>();
    let output_glue_domain_carrier = Type::new_box_pair(&output_glue0.domain_carrier, &output_glue1.domain_carrier);
    let output_glue_measure_glue = output_glue0.measure_glue.clone();
    let output_glue = FfiMeasureGlue::<FfiDomain, FfiMeasure>::new_explicit(output_glue_domain_type, output_glue_domain_carrier, output_glue_measure_glue);

    FfiResult::Ok(util::into_raw(
        FfiMeasurement::new(input_glue0.clone(), output_glue, measurement)))
}