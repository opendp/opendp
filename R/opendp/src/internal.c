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


SEXP internal___binary_search(
    SEXP predicate, SEXP bounds, SEXP return_sign, SEXP T, SEXP T_predicate, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(predicate);
    PROTECT(bounds);
    PROTECT(return_sign);
    PROTECT(T);
    PROTECT(T_predicate);
    PROTECT(T_bounds);
    PROTECT(log);

    char * c_T = rt_to_string(T);
    char * c_T_predicate = rt_to_string(T_predicate);
    char * c_T_bounds = rt_to_string(T_bounds);
    CallbackFn c_predicate = sexp_to_callbackfn(predicate, c_T_predicate);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    bool c_return_sign = asLogical(return_sign);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_internal___binary_search(&c_predicate, c_bounds, c_return_sign, c_T);

    callbackfn_release(&c_predicate);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP internal___exponential_bounds_search(
    SEXP predicate, SEXP T, SEXP T_predicate, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(predicate);
    PROTECT(T);
    PROTECT(T_predicate);
    PROTECT(log);

    char * c_T = rt_to_string(T);
    char * c_T_predicate = rt_to_string(T_predicate);
    CallbackFn c_predicate = sexp_to_callbackfn(predicate, c_T_predicate);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_internal___exponential_bounds_search(&c_predicate, c_T);

    callbackfn_release(&c_predicate);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP internal___extrinsic_distance(
    SEXP identifier, SEXP descriptor, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(identifier);
    PROTECT(descriptor);
    PROTECT(log);

    char * c_identifier = Rf_isNull(identifier) ? NULL : (char *)CHAR(STRING_ELT(identifier, 0));
    ExtrinsicObject * c_descriptor = sexp_to_extrinsicobjectptr(descriptor);

    // Call library function.
    FfiResult_____AnyMetric _result = opendp_internal___extrinsic_distance(c_identifier, c_descriptor);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMetric)
        return(extract_error(_result.err));
    AnyMetric* _return_value = _result.ok;
    return(anymetricptr_to_sexp(_return_value, log));
}


SEXP internal___extrinsic_divergence(
    SEXP identifier, SEXP descriptor, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(identifier);
    PROTECT(descriptor);
    PROTECT(log);

    char * c_identifier = Rf_isNull(identifier) ? NULL : (char *)CHAR(STRING_ELT(identifier, 0));
    ExtrinsicObject * c_descriptor = sexp_to_extrinsicobjectptr(descriptor);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_internal___extrinsic_divergence(c_identifier, c_descriptor);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP internal___extrinsic_domain(
    SEXP identifier, SEXP member, SEXP descriptor, SEXP T_member, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(identifier);
    PROTECT(member);
    PROTECT(descriptor);
    PROTECT(T_member);
    PROTECT(log);

    char * c_T_member = rt_to_string(T_member);
    char * c_identifier = Rf_isNull(identifier) ? NULL : (char *)CHAR(STRING_ELT(identifier, 0));
    CallbackFn c_member = sexp_to_callbackfn(member, c_T_member);
    ExtrinsicObject * c_descriptor = sexp_to_extrinsicobjectptr(descriptor);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_internal___extrinsic_domain(c_identifier, &c_member, c_descriptor);

    callbackfn_release(&c_member);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP internal___make_measurement(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP function, SEXP privacy_map, SEXP TO, SEXP T_function, SEXP T_privacy_map, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(function);
    PROTECT(privacy_map);
    PROTECT(TO);
    PROTECT(T_function);
    PROTECT(T_privacy_map);
    PROTECT(log);

    char * c_TO = rt_to_string(TO);
    char * c_T_function = rt_to_string(T_function);
    char * c_T_privacy_map = rt_to_string(T_privacy_map);
    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    CallbackFn c_function = sexp_to_callbackfn(function, c_T_function);
    CallbackFn c_privacy_map = sexp_to_callbackfn(privacy_map, c_T_privacy_map);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_internal___make_measurement(c_input_domain, c_input_metric, c_output_measure, &c_function, &c_privacy_map, c_TO);

    callbackfn_release(&c_function);
    callbackfn_release(&c_privacy_map);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP internal___make_transformation(
    SEXP input_domain, SEXP input_metric, SEXP output_domain, SEXP output_metric, SEXP function, SEXP stability_map, SEXP T_function, SEXP T_stability_map, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_domain);
    PROTECT(output_metric);
    PROTECT(function);
    PROTECT(stability_map);
    PROTECT(T_function);
    PROTECT(T_stability_map);
    PROTECT(log);

    char * c_T_function = rt_to_string(T_function);
    char * c_T_stability_map = rt_to_string(T_stability_map);
    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyDomain * c_output_domain = sexp_to_anydomainptr(output_domain);
    AnyMetric * c_output_metric = sexp_to_anymetricptr(output_metric);
    CallbackFn c_function = sexp_to_callbackfn(function, c_T_function);
    CallbackFn c_stability_map = sexp_to_callbackfn(stability_map, c_T_stability_map);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_internal___make_transformation(c_input_domain, c_input_metric, c_output_domain, c_output_metric, &c_function, &c_stability_map);

    callbackfn_release(&c_function);
    callbackfn_release(&c_stability_map);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP internal___new_pure_function(
    SEXP function, SEXP TO, SEXP T_function, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(function);
    PROTECT(TO);
    PROTECT(T_function);
    PROTECT(log);

    char * c_TO = rt_to_string(TO);
    char * c_T_function = rt_to_string(T_function);
    CallbackFn c_function = sexp_to_callbackfn(function, c_T_function);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_internal___new_pure_function(&c_function, c_TO);

    callbackfn_release(&c_function);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}

