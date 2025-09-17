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
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP total_limit, SEXP value_limit, SEXP size_factor, SEXP alpha, SEXP CI, SEXP T_value_limit, SEXP T_size_factor, SEXP T_alpha, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(total_limit);
    PROTECT(value_limit);
    PROTECT(size_factor);
    PROTECT(alpha);
    PROTECT(CI);
    PROTECT(T_value_limit);
    PROTECT(T_size_factor);
    PROTECT(T_alpha);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    void * c_total_limit = sexp_to_voidptr(total_limit, CI);
    void * c_value_limit = sexp_to_voidptr(value_limit, T_value_limit);
    void * c_size_factor = sexp_to_voidptr(size_factor, T_size_factor);
    void * c_alpha = sexp_to_voidptr(alpha, T_alpha);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_alp_queryable(c_input_domain, c_input_metric, c_scale, c_total_limit, c_value_limit, c_size_factor, c_alpha);

    UNPROTECT(12);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_canonical_noise(
    SEXP input_domain, SEXP input_metric, SEXP d_in, SEXP d_out, SEXP T_d_out, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(d_in);
    PROTECT(d_out);
    PROTECT(T_d_out);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_d_in = Rf_asReal(d_in);
    AnyObject * c_d_out = sexp_to_anyobjectptr(d_out, T_d_out);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_canonical_noise(c_input_domain, c_input_metric, c_d_in, c_d_out);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_gaussian(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP MO, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(k);
    PROTECT(MO);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_gaussian(c_input_domain, c_input_metric, c_scale, c_k, c_MO);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_gaussian_threshold(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP threshold, SEXP k, SEXP MO, SEXP TV, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(threshold);
    PROTECT(k);
    PROTECT(MO);
    PROTECT(TV);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    void * c_threshold = sexp_to_voidptr(threshold, TV);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_gaussian_threshold(c_input_domain, c_input_metric, c_scale, c_threshold, c_k, c_MO);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_geometric(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP bounds, SEXP MO, SEXP T, SEXP OptionT, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(bounds);
    PROTECT(MO);
    PROTECT(T);
    PROTECT(OptionT);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, OptionT);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_geometric(c_input_domain, c_input_metric, c_scale, c_bounds, c_MO);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_laplace(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP k, SEXP MO, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(k);
    PROTECT(MO);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_laplace(c_input_domain, c_input_metric, c_scale, c_k, c_MO);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_laplace_threshold(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP threshold, SEXP k, SEXP MO, SEXP TV, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(threshold);
    PROTECT(k);
    PROTECT(MO);
    PROTECT(TV);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    void * c_threshold = sexp_to_voidptr(threshold, TV);
    void * c_k = sexp_to_voidptr(k, T_k);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_laplace_threshold(c_input_domain, c_input_metric, c_scale, c_threshold, c_k, c_MO);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_noise(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP scale, SEXP k, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(scale);
    PROTECT(k);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    double c_scale = Rf_asReal(scale);
    void * c_k = sexp_to_voidptr(k, T_k);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_noise(c_input_domain, c_input_metric, c_output_measure, c_scale, c_k);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_noise_threshold(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP scale, SEXP threshold, SEXP k, SEXP TV, SEXP T_k, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(scale);
    PROTECT(threshold);
    PROTECT(k);
    PROTECT(TV);
    PROTECT(T_k);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    double c_scale = Rf_asReal(scale);
    void * c_threshold = sexp_to_voidptr(threshold, TV);
    void * c_k = sexp_to_voidptr(k, T_k);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_noise_threshold(c_input_domain, c_input_metric, c_output_measure, c_scale, c_threshold, c_k);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_noisy_max(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP scale, SEXP negate, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(scale);
    PROTECT(negate);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    double c_scale = Rf_asReal(scale);
    bool c_negate = asLogical(negate);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_noisy_max(c_input_domain, c_input_metric, c_output_measure, c_scale, c_negate);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_noisy_top_k(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP k, SEXP scale, SEXP negate, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(k);
    PROTECT(scale);
    PROTECT(negate);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    size_t c_k = (size_t)Rf_asInteger(k);
    double c_scale = Rf_asReal(scale);
    bool c_negate = asLogical(negate);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_noisy_top_k(c_input_domain, c_input_metric, c_output_measure, c_k, c_scale, c_negate);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_private_quantile(
    SEXP input_domain, SEXP input_metric, SEXP output_measure, SEXP candidates, SEXP alpha, SEXP scale, SEXP T, SEXP T_candidates, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(output_measure);
    PROTECT(candidates);
    PROTECT(alpha);
    PROTECT(scale);
    PROTECT(T);
    PROTECT(T_candidates);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyMeasure * c_output_measure = sexp_to_anymeasureptr(output_measure);
    AnyObject * c_candidates = sexp_to_anyobjectptr(candidates, T_candidates);
    double c_alpha = Rf_asReal(alpha);
    double c_scale = Rf_asReal(scale);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_private_quantile(c_input_domain, c_input_metric, c_output_measure, c_candidates, c_alpha, c_scale);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_randomized_response(
    SEXP categories, SEXP prob, SEXP T, SEXP T_categories, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(categories);
    PROTECT(prob);
    PROTECT(T);
    PROTECT(T_categories);
    PROTECT(log);

    AnyObject * c_categories = sexp_to_anyobjectptr(categories, T_categories);
    double c_prob = Rf_asReal(prob);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_randomized_response(c_categories, c_prob, c_T);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_randomized_response_bitvec(
    SEXP input_domain, SEXP input_metric, SEXP f, SEXP constant_time, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(f);
    PROTECT(constant_time);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_f = Rf_asReal(f);
    bool c_constant_time = asLogical(constant_time);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_randomized_response_bitvec(c_input_domain, c_input_metric, c_f, c_constant_time);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_randomized_response_bool(
    SEXP prob, SEXP constant_time, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(prob);
    PROTECT(constant_time);
    PROTECT(log);

    double c_prob = Rf_asReal(prob);
    bool c_constant_time = asLogical(constant_time);

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_randomized_response_bool(c_prob, c_constant_time);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}


SEXP measurements__make_report_noisy_max_gumbel(
    SEXP input_domain, SEXP input_metric, SEXP scale, SEXP optimize, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(scale);
    PROTECT(optimize);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    double c_scale = Rf_asReal(scale);
    char * c_optimize = (char *)CHAR(STRING_ELT(optimize, 0));

    // Call library function.
    FfiResult_____AnyMeasurement _result = opendp_measurements__make_report_noisy_max_gumbel(c_input_domain, c_input_metric, c_scale, c_optimize);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyMeasurement)
        return(extract_error(_result.err));
    AnyMeasurement* _return_value = _result.ok;
    return(anymeasurementptr_to_sexp(_return_value, log));
}

