// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

#include "convert.h"
#include "convert_elements.h"
#include "Ropendp.h"
#include "opendp.h"
#include "opendp_extras.h"


SEXP core__function_eval(
    SEXP this, SEXP arg, SEXP TI, SEXP T_arg, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(arg);
    PROTECT(TI);
    PROTECT(T_arg);
    PROTECT(log);

    AnyFunction * c_this = sexp_to_anyfunctionptr(this);
    AnyObject * c_arg = sexp_to_anyobjectptr(arg, T_arg);
    char * c_TI = (char *)CHAR(STRING_ELT(TI, 0));

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__function_eval(c_this, c_arg, c_TI);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__measurement_check(
    SEXP measurement, SEXP distance_in, SEXP distance_out, SEXP T_distance_in, SEXP T_distance_out, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(distance_in);
    PROTECT(distance_out);
    PROTECT(T_distance_in);
    PROTECT(T_distance_out);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);
    AnyObject * c_distance_in = sexp_to_anyobjectptr(distance_in, T_distance_in);
    AnyObject * c_distance_out = sexp_to_anyobjectptr(distance_out, T_distance_out);

    // Call library function.
    FfiResult_____c_bool _result = opendp_core__measurement_check(c_measurement, c_distance_in, c_distance_out);

    UNPROTECT(6);
    if(_result.tag == Err_____c_bool)
        return(extract_error(_result.err));
    c_bool* _return_value = _result.ok;
    return(ScalarLogical(*(bool *)_return_value));
}


SEXP core__measurement_function(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_core__measurement_function(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}


SEXP core__measurement_input_carrier_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__measurement_input_carrier_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__measurement_input_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__measurement_input_distance_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__measurement_input_domain(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_core__measurement_input_domain(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP core__measurement_input_metric(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_core__measurement_input_metric(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP core__measurement_invoke(
    SEXP this, SEXP arg, SEXP T_arg, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(arg);
    PROTECT(T_arg);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);
    AnyObject * c_arg = sexp_to_anyobjectptr(arg, T_arg);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__measurement_invoke(c_this, c_arg);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__measurement_map(
    SEXP measurement, SEXP distance_in, SEXP T_distance_in, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measurement);
    PROTECT(distance_in);
    PROTECT(T_distance_in);
    PROTECT(log);

    AnyMeasurement * c_measurement = sexp_to_anymeasurementptr(measurement);
    AnyObject * c_distance_in = sexp_to_anyobjectptr(distance_in, T_distance_in);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__measurement_map(c_measurement, c_distance_in);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__measurement_output_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__measurement_output_distance_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__measurement_output_measure(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasurement * c_this = sexp_to_anymeasurementptr(this);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_core__measurement_output_measure(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP core__odometer_input_carrier_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyOdometer * c_this = sexp_to_anyodometerptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__odometer_input_carrier_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__odometer_input_domain(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyOdometer * c_this = sexp_to_anyodometerptr(this);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_core__odometer_input_domain(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP core__odometer_input_metric(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyOdometer * c_this = sexp_to_anyodometerptr(this);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_core__odometer_input_metric(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP core__odometer_invoke(
    SEXP this, SEXP arg, SEXP T_arg, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(arg);
    PROTECT(T_arg);
    PROTECT(log);

    AnyOdometer * c_this = sexp_to_anyodometerptr(this);
    AnyObject * c_arg = sexp_to_anyobjectptr(arg, T_arg);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__odometer_invoke(c_this, c_arg);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__odometer_output_measure(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyOdometer * c_this = sexp_to_anyodometerptr(this);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_core__odometer_output_measure(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP core__odometer_queryable_invoke(
    SEXP queryable, SEXP query, SEXP T_query, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(queryable);
    PROTECT(query);
    PROTECT(T_query);
    PROTECT(log);

    AnyObject * c_queryable = sexp_to_anyobjectptr(queryable, R_NilValue);
    AnyObject * c_query = sexp_to_anyobjectptr(query, T_query);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__odometer_queryable_invoke(c_queryable, c_query);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__odometer_queryable_invoke_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__odometer_queryable_invoke_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__odometer_queryable_privacy_loss(
    SEXP queryable, SEXP d_in, SEXP T_d_in, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(queryable);
    PROTECT(d_in);
    PROTECT(T_d_in);
    PROTECT(log);

    AnyObject * c_queryable = sexp_to_anyobjectptr(queryable, R_NilValue);
    AnyObject * c_d_in = sexp_to_anyobjectptr(d_in, T_d_in);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__odometer_queryable_privacy_loss(c_queryable, c_d_in);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__odometer_queryable_privacy_loss_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__odometer_queryable_privacy_loss_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__queryable_eval(
    SEXP queryable, SEXP query, SEXP T_query, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(queryable);
    PROTECT(query);
    PROTECT(T_query);
    PROTECT(log);

    AnyObject * c_queryable = sexp_to_anyobjectptr(queryable, R_NilValue);
    AnyObject * c_query = sexp_to_anyobjectptr(query, T_query);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__queryable_eval(c_queryable, c_query);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__queryable_query_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__queryable_query_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__transformation_check(
    SEXP transformation, SEXP distance_in, SEXP distance_out, SEXP T_distance_in, SEXP T_distance_out, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(transformation);
    PROTECT(distance_in);
    PROTECT(distance_out);
    PROTECT(T_distance_in);
    PROTECT(T_distance_out);
    PROTECT(log);

    AnyTransformation * c_transformation = sexp_to_anytransformationptr(transformation);
    AnyObject * c_distance_in = sexp_to_anyobjectptr(distance_in, T_distance_in);
    AnyObject * c_distance_out = sexp_to_anyobjectptr(distance_out, T_distance_out);

    // Call library function.
    FfiResult_____c_bool _result = opendp_core__transformation_check(c_transformation, c_distance_in, c_distance_out);

    UNPROTECT(6);
    if(_result.tag == Err_____c_bool)
        return(extract_error(_result.err));
    c_bool* _return_value = _result.ok;
    return(ScalarLogical(*(bool *)_return_value));
}


SEXP core__transformation_function(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_core__transformation_function(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}


SEXP core__transformation_input_carrier_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__transformation_input_carrier_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__transformation_input_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__transformation_input_distance_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__transformation_input_domain(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_core__transformation_input_domain(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP core__transformation_input_metric(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_core__transformation_input_metric(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP core__transformation_invoke(
    SEXP this, SEXP arg, SEXP T_arg, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(arg);
    PROTECT(T_arg);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);
    AnyObject * c_arg = sexp_to_anyobjectptr(arg, T_arg);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__transformation_invoke(c_this, c_arg);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__transformation_map(
    SEXP transformation, SEXP distance_in, SEXP T_distance_in, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(transformation);
    PROTECT(distance_in);
    PROTECT(T_distance_in);
    PROTECT(log);

    AnyTransformation * c_transformation = sexp_to_anytransformationptr(transformation);
    AnyObject * c_distance_in = sexp_to_anyobjectptr(distance_in, T_distance_in);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_core__transformation_map(c_transformation, c_distance_in);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP core__transformation_output_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_core__transformation_output_distance_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP core__transformation_output_domain(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_core__transformation_output_domain(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP core__transformation_output_metric(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyTransformation * c_this = sexp_to_anytransformationptr(this);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_core__transformation_output_metric(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}

