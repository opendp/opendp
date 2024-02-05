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


SEXP measures__fixed_smoothed_max_divergence(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    PROTECT(log);

    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__fixed_smoothed_max_divergence(c_T);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}


SEXP measures__max_divergence(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    PROTECT(log);

    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__max_divergence(c_T);

    UNPROTECT(2);
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


SEXP measures__smoothed_max_divergence(
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    PROTECT(log);

    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__smoothed_max_divergence(c_T);

    UNPROTECT(2);
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
    SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(T);
    PROTECT(log);

    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMeasure _result = opendp_measures__zero_concentrated_divergence(c_T);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyMeasure)
        return(extract_error(_result.err));
    AnyMeasure* _return_value = _result.ok;
    return(anymeasureptr_to_sexp(_return_value, log));
}

