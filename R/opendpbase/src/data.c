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


SEXP data__extrinsic_object_free(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    ExtrinsicObject * c_this = "UNKNOWN TYPE: ExtrinsicObject *";

    // Call library function.
    FfiResult_____c_void _result = opendp_data__extrinsic_object_free(c_this);

    UNPROTECT(1);
    if(_result.tag == Err_____c_void)
        return(extract_error(_result.err));
    c_void* _return_value = _result.ok;
    return(voidptr_to_sexp(_return_value, ScalarString(mkChar("void"))));
}


SEXP data__fill_bytes(
    SEXP ptr, SEXP len, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(ptr);
    PROTECT(len);
    uint8_t * c_ptr = "UNKNOWN TYPE: uint8_t *";
    size_t c_len = (size_t)Rf_asInteger(len);

    // Call library function.
    c_bool _result = opendp_data__fill_bytes(c_ptr, c_len);

    UNPROTECT(2);
    return(ScalarInteger(_result));
}


SEXP data__object_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_data__object_type(c_this);

    UNPROTECT(1);
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
    AnyObject * c_curve = sexp_to_anyobjectptr(curve, R_NilValue);
    AnyObject * c_delta = sexp_to_anyobjectptr(delta, T_delta);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_data__smd_curve_epsilon(c_curve, c_delta);

    UNPROTECT(3);
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
    AnyObject * c_this = sexp_to_anyobjectptr(this, R_NilValue);

    // Call library function.
    FfiResult_____c_char _result = opendp_data__to_string(c_this);

    UNPROTECT(1);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}

