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


SEXP domains___atom_domain_get_bounds_closed(
    SEXP domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(domain);
    PROTECT(log);

    AnyDomain * c_domain = sexp_to_anydomainptr(domain);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_domains___atom_domain_get_bounds_closed(c_domain);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP domains___atom_domain_nan(
    SEXP domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(domain);
    PROTECT(log);

    AnyDomain * c_domain = sexp_to_anydomainptr(domain);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_domains___atom_domain_nan(c_domain);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP domains___domain_equal(
    SEXP left, SEXP right, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(left);
    PROTECT(right);
    PROTECT(log);

    AnyDomain * c_left = sexp_to_anydomainptr(left);
    AnyDomain * c_right = sexp_to_anydomainptr(right);

    // Call library function.
    FfiResult_____c_bool _result = opendp_domains___domain_equal(c_left, c_right);

    UNPROTECT(3);
    if(_result.tag == Err_____c_bool)
        return(extract_error(_result.err));
    c_bool* _return_value = _result.ok;
    return(ScalarLogical(*(bool *)_return_value));
}


SEXP domains___option_domain_get_element_domain(
    SEXP option_domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(option_domain);
    PROTECT(log);

    AnyDomain * c_option_domain = sexp_to_anydomainptr(option_domain);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains___option_domain_get_element_domain(c_option_domain);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains___vector_domain_get_element_domain(
    SEXP vector_domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(vector_domain);
    PROTECT(log);

    AnyDomain * c_vector_domain = sexp_to_anydomainptr(vector_domain);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains___vector_domain_get_element_domain(c_vector_domain);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains___vector_domain_get_size(
    SEXP vector_domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(vector_domain);
    PROTECT(log);

    AnyDomain * c_vector_domain = sexp_to_anydomainptr(vector_domain);

    // Call library function.
    FfiResult_____AnyObject _result = opendp_domains___vector_domain_get_size(c_vector_domain);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyObject)
        return(extract_error(_result.err));
    AnyObject* _return_value = _result.ok;
    return(anyobjectptr_to_sexp(_return_value));
}


SEXP domains__atom_domain(
    SEXP bounds, SEXP nan, SEXP T, SEXP T_bounds, SEXP T_nan, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(bounds);
    PROTECT(nan);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(T_nan);
    PROTECT(log);

    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    AnyObject * c_nan = sexp_to_anyobjectptr(nan, T_nan);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains__atom_domain(c_bounds, c_nan, c_T);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains__bitvector_domain(
    SEXP max_weight, SEXP T_max_weight, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(max_weight);
    PROTECT(T_max_weight);
    PROTECT(log);

    AnyObject * c_max_weight = sexp_to_anyobjectptr(max_weight, T_max_weight);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains__bitvector_domain(c_max_weight);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains__domain_carrier_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyDomain * c_this = sexp_to_anydomainptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_domains__domain_carrier_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP domains__domain_debug(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyDomain * c_this = sexp_to_anydomainptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_domains__domain_debug(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP domains__domain_type(
    SEXP this, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(log);

    AnyDomain * c_this = sexp_to_anydomainptr(this);

    // Call library function.
    FfiResult_____c_char _result = opendp_domains__domain_type(c_this);

    UNPROTECT(2);
    if(_result.tag == Err_____c_char)
        return(extract_error(_result.err));
    c_char* _return_value = _result.ok;
    return(ScalarString(mkChar(_return_value)));
}


SEXP domains__map_domain(
    SEXP key_domain, SEXP value_domain, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(key_domain);
    PROTECT(value_domain);
    PROTECT(log);

    AnyDomain * c_key_domain = sexp_to_anydomainptr(key_domain);
    AnyDomain * c_value_domain = sexp_to_anydomainptr(value_domain);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains__map_domain(c_key_domain, c_value_domain);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains__member(
    SEXP this, SEXP val, SEXP T_val, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(this);
    PROTECT(val);
    PROTECT(T_val);
    PROTECT(log);

    AnyDomain * c_this = sexp_to_anydomainptr(this);
    AnyObject * c_val = sexp_to_anyobjectptr(val, T_val);

    // Call library function.
    FfiResult_____c_bool _result = opendp_domains__member(c_this, c_val);

    UNPROTECT(4);
    if(_result.tag == Err_____c_bool)
        return(extract_error(_result.err));
    c_bool* _return_value = _result.ok;
    return(ScalarLogical(*(bool *)_return_value));
}


SEXP domains__option_domain(
    SEXP element_domain, SEXP D, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(element_domain);
    PROTECT(D);
    PROTECT(log);

    AnyDomain * c_element_domain = sexp_to_anydomainptr(element_domain);
    char * c_D = rt_to_string(D);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains__option_domain(c_element_domain, c_D);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}


SEXP domains__vector_domain(
    SEXP atom_domain, SEXP size, SEXP T_size, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(atom_domain);
    PROTECT(size);
    PROTECT(T_size);
    PROTECT(log);

    AnyDomain * c_atom_domain = sexp_to_anydomainptr(atom_domain);
    AnyObject * c_size = sexp_to_anyobjectptr(size, T_size);

    // Call library function.
    FfiResult_____AnyDomain _result = opendp_domains__vector_domain(c_atom_domain, c_size);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyDomain)
        return(extract_error(_result.err));
    AnyDomain* _return_value = _result.ok;
    return(anydomainptr_to_sexp(_return_value, log));
}

