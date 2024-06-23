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


SEXP data__erf_inv(
    SEXP value, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(value);
    PROTECT(log);

    double c_value = Rf_asReal(value);

    // Call library function.
    double _result = opendp_data__erf_inv(c_value);

    UNPROTECT(2);
    return(ScalarReal(*(double *)_result));
}


SEXP data__object_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_data__object_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP data__smd_curve_epsilon(
    SEXP curve, SEXP delta, SEXP T_delta, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(curve);
    PROTECT(delta);
    PROTECT(T_delta);
    PROTECT(log);

    AnyObject * c_curve = sexp_to_anyobjectptr(curve, R_NilValue);
    AnyObject * c_delta = sexp_to_anyobjectptr(delta, T_delta);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_data__smd_curve_epsilon(c_curve, c_delta);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP data__to_string(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_data__to_string(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}

