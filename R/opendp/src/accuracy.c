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


SEXP accuracy__accuracy_to_discrete_gaussian_scale(
    SEXP accuracy, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(accuracy);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_accuracy = sexp_to_voidptr(accuracy, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__accuracy_to_discrete_gaussian_scale(c_accuracy, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__accuracy_to_discrete_laplacian_scale(
    SEXP accuracy, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(accuracy);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_accuracy = sexp_to_voidptr(accuracy, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__accuracy_to_discrete_laplacian_scale(c_accuracy, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__accuracy_to_gaussian_scale(
    SEXP accuracy, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(accuracy);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_accuracy = sexp_to_voidptr(accuracy, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__accuracy_to_gaussian_scale(c_accuracy, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__accuracy_to_laplacian_scale(
    SEXP accuracy, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(accuracy);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_accuracy = sexp_to_voidptr(accuracy, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__accuracy_to_laplacian_scale(c_accuracy, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__discrete_gaussian_scale_to_accuracy(
    SEXP scale, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(scale);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_scale = sexp_to_voidptr(scale, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__discrete_gaussian_scale_to_accuracy(c_scale, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__discrete_laplacian_scale_to_accuracy(
    SEXP scale, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(scale);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_scale = sexp_to_voidptr(scale, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__discrete_laplacian_scale_to_accuracy(c_scale, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__gaussian_scale_to_accuracy(
    SEXP scale, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(scale);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_scale = sexp_to_voidptr(scale, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__gaussian_scale_to_accuracy(c_scale, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP accuracy__laplacian_scale_to_accuracy(
    SEXP scale, SEXP alpha, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(scale);
    PROTECT(alpha);
    PROTECT(T);
    PROTECT(log);

    void * c_scale = sexp_to_voidptr(scale, T);
    void * c_alpha = sexp_to_voidptr(alpha, T);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_accuracy__laplacian_scale_to_accuracy(c_scale, c_alpha, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}

