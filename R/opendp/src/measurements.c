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


SEXP measurements__make_alp_queryable(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP total_limit, SEXP value_limit, SEXP size_factor, SEXP alpha, SEXP CO, SEXP CI, SEXP T_value_limit, SEXP T_size_factor, SEXP T_alpha, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(total_limit);
    PROTECT(value_limit);
    PROTECT(size_factor);
    PROTECT(alpha);
    PROTECT(CO);
    PROTECT(CI);
    PROTECT(T_value_limit);
    PROTECT(T_size_factor);
    PROTECT(T_alpha);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_scale = sexp_to_voidptr(scale, CO);
    void * c_total_limit = sexp_to_voidptr(total_limit, CI);
    void * c_value_limit = sexp_to_voidptr(value_limit, T_value_limit);
    void * c_size_factor = sexp_to_voidptr(size_factor, T_size_factor);
    void * c_alpha = sexp_to_voidptr(alpha, T_alpha);
    char * c_CO = rt_to_string(CO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_alp_queryable(c_input_domain, c_input_metric, c_scale, c_total_limit, c_value_limit, c_size_factor, c_alpha, c_CO);

    UNPROTECT(13);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_gaussian(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP MO, SEXP QO, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(k);
    PROTECT(MO);
    PROTECT(QO);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_scale = sexp_to_voidptr(scale, QO);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_gaussian(c_input_domain, c_input_metric, c_scale, c_k, c_MO);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_geometric(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP bounds, SEXP QO, SEXP T, SEXP OptionT, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(bounds);
    PROTECT(QO);
    PROTECT(T);
    PROTECT(OptionT);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_scale = sexp_to_voidptr(scale, QO);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, OptionT);
    char * c_QO = rt_to_string(QO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_geometric(c_input_domain, c_input_metric, c_scale, c_bounds, c_QO);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_laplace(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP QO, SEXP T_scale, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(k);
    PROTECT(QO);
    PROTECT(T_scale);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_scale = sexp_to_voidptr(scale, T_scale);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_QO = rt_to_string(QO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_laplace(c_input_domain, c_input_metric, c_scale, c_k, c_QO);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_laplace_threshold(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP threshold, SEXP k, SEXP TV, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(threshold);
    PROTECT(k);
    PROTECT(TV);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_scale = sexp_to_voidptr(scale, TV);
    void * c_threshold = sexp_to_voidptr(threshold, TV);
    uint32_t c_k = (unsigned int)Rf_asInteger(k);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_laplace_threshold(c_input_domain, c_input_metric, c_scale, c_threshold, c_k);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_randomized_response(
    SEXP categories, SEXP prob, SEXP constant_time, SEXP T, SEXP QO, SEXP T_categories, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(categories);
    PROTECT(prob);
    PROTECT(constant_time);
    PROTECT(T);
    PROTECT(QO);
    PROTECT(T_categories);
    PROTECT(log);

    AnyObject * c_categories = sexp_to_anyobjectptr(categories, T_categories);
    void * c_prob = sexp_to_voidptr(prob, QO);
    bool c_constant_time = asLogical(constant_time);
    char * c_T = rt_to_string(T);
    char * c_QO = rt_to_string(QO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_randomized_response(c_categories, c_prob, c_constant_time, c_T, c_QO);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_randomized_response_bool(
    SEXP prob, SEXP constant_time, SEXP QO, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(prob);
    PROTECT(constant_time);
    PROTECT(QO);
    PROTECT(log);

    void * c_prob = sexp_to_voidptr(prob, QO);
    bool c_constant_time = asLogical(constant_time);
    char * c_QO = rt_to_string(QO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_randomized_response_bool(c_prob, c_constant_time, c_QO);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_report_noisy_max_gumbel(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP optimize, SEXP QO, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(optimize);
    PROTECT(QO);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_scale = sexp_to_anyobjectptr(scale, QO);
    char * c_optimize = (char *)CHAR(STRING_ELT(optimize, 0));
    char * c_QO = rt_to_string(QO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_report_noisy_max_gumbel(c_input_domain, c_input_metric, c_scale, c_optimize, c_QO);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_tulap(
    SEXP input_domain, SEXP input_metric, SEXP epsilon, SEXP delta, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(epsilon);
    PROTECT(delta);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_epsilon = Rf_asReal(epsilon);
    double c_delta = Rf_asReal(delta);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_tulap(c_input_domain, c_input_metric, c_epsilon, c_delta);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}

