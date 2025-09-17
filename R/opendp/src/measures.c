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


SEXP measures___approximate_divergence_get_inner_measure(
    SEXP privacy_measure, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(privacy_measure);
    PROTECT(log);

    AnyMeasure * c_privacy_measure = sexp_to_anymeasureptr(privacy_measure);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures___approximate_divergence_get_inner_measure(c_privacy_measure);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures___measure_equal(
    SEXP left, SEXP right, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(left);
    PROTECT(right);
    PROTECT(log);

    AnyMeasure * c_left = sexp_to_anymeasureptr(left);
    AnyMeasure * c_right = sexp_to_anymeasureptr(right);

    // Call library function.
    FfiResult_____c_bool _result = opendp_measures___measure_equal(c_left, c_right);

    UNPROTECT(3);
    if(_result.tag == Err_____c_bool)
        return(extract_error(_result.err));
    c_bool* _return_value = _result.ok;
    return(ScalarLogical(*(bool *)_return_value));
}


SEXP measures__approximate(
    SEXP measure, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(measure);
    PROTECT(log);

    AnyMeasure * c_measure = sexp_to_anymeasureptr(measure);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__approximate(c_measure);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__fixed_smoothed_max_divergence(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__fixed_smoothed_max_divergence();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__max_divergence(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__max_divergence();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__measure_debug(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasure * c_this = sexp_to_anymeasureptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_measures__measure_debug(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP measures__measure_distance_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasure * c_this = sexp_to_anymeasureptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_measures__measure_distance_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP measures__measure_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyMeasure * c_this = sexp_to_anymeasureptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_measures__measure_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP measures__renyi_divergence(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__renyi_divergence();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__smoothed_max_divergence(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__smoothed_max_divergence();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__user_divergence(
    SEXP descriptor, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(descriptor);
    PROTECT(log);

    char * c_descriptor = (char *)CHAR(STRING_ELT(descriptor, 0));

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__user_divergence(c_descriptor);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__zero_concentrated_divergence(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__zero_concentrated_divergence();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}

