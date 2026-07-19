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


SEXP data__privacy_profile_delta(
    SEXP curve, SEXP epsilon, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(curve);
    PROTECT(epsilon);
    PROTECT(log);

    AnyObject * c_curve = sexp_to_anyobjectptr(curve, R_NilValue);
    double c_epsilon = Rf_asReal(epsilon);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_data__privacy_profile_delta(c_curve, c_epsilon);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP data__privacy_profile_epsilon(
    SEXP profile, SEXP delta, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(profile);
    PROTECT(delta);
    PROTECT(log);

    AnyObject * c_profile = sexp_to_anyobjectptr(profile, R_NilValue);
    double c_delta = Rf_asReal(delta);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_data__privacy_profile_epsilon(c_profile, c_delta);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}

