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


SEXP transformations__choose_branching_factor(
    SEXP size_guess, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size_guess);
    PROTECT(log);

    uint32_t c_size_guess = (unsigned int)Rf_asInteger(size_guess);

    // Call library function.
    uint32_t _result = opendp_transformations__choose_branching_factor(c_size_guess);

    UNPROTECT(2);
    return(ScalarInteger((int)_result));
}


SEXP transformations__make_b_ary_tree(
    SEXP input_domain, SEXP input_metric, SEXP leaf_count, SEXP branching_factor, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(leaf_count);
    PROTECT(branching_factor);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    uint32_t c_leaf_count = (unsigned int)Rf_asInteger(leaf_count);
    uint32_t c_branching_factor = (unsigned int)Rf_asInteger(branching_factor);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_b_ary_tree(c_input_domain, c_input_metric, c_leaf_count, c_branching_factor);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_bounded_float_checked_sum(
    SEXP size_limit, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size_limit);
    PROTECT(bounds);
    PROTECT(S);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size_limit = (size_t)Rf_asInteger(size_limit);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_bounded_float_checked_sum(c_size_limit, c_bounds, c_S);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_bounded_float_ordered_sum(
    SEXP size_limit, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size_limit);
    PROTECT(bounds);
    PROTECT(S);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size_limit = (size_t)Rf_asInteger(size_limit);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_bounded_float_ordered_sum(c_size_limit, c_bounds, c_S);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_bounded_int_monotonic_sum(
    SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_bounded_int_monotonic_sum(c_bounds, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_bounded_int_ordered_sum(
    SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_bounded_int_ordered_sum(c_bounds, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_bounded_int_split_sum(
    SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_bounded_int_split_sum(c_bounds, c_T);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_cast(
    SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(TOA);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_cast(c_input_domain, c_input_metric, c_TOA);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_cast_default(
    SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP TIA, SEXP M, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(TOA);
    PROTECT(TIA);
    PROTECT(M);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_cast_default(c_input_domain, c_input_metric, c_TOA);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_cast_inherent(
    SEXP input_domain, SEXP input_metric, SEXP TOA, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(TOA);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_cast_inherent(c_input_domain, c_input_metric, c_TOA);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_cdf(
    SEXP TA, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(TA);
    PROTECT(log);

    char * c_TA = rt_to_string(TA);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_transformations__make_cdf(c_TA);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}


SEXP transformations__make_clamp(
    SEXP input_domain, SEXP input_metric, SEXP bounds, SEXP TA, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(bounds);
    PROTECT(TA);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_clamp(c_input_domain, c_input_metric, c_bounds);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_consistent_b_ary_tree(
    SEXP branching_factor, SEXP TIA, SEXP TOA, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(branching_factor);
    PROTECT(TIA);
    PROTECT(TOA);
    PROTECT(log);

    uint32_t c_branching_factor = (unsigned int)Rf_asInteger(branching_factor);
    char * c_TIA = rt_to_string(TIA);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_transformations__make_consistent_b_ary_tree(c_branching_factor, c_TIA, c_TOA);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}


SEXP transformations__make_count(
    SEXP input_domain, SEXP input_metric, SEXP TO, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(TO);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_TO = rt_to_string(TO);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_count(c_input_domain, c_input_metric, c_TO);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_count_by(
    SEXP input_domain, SEXP input_metric, SEXP MO, SEXP TV, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(MO);
    PROTECT(TV);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_MO = rt_to_string(MO);
    char * c_TV = rt_to_string(TV);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_count_by(c_input_domain, c_input_metric, c_MO, c_TV);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_count_by_categories(
    SEXP input_domain, SEXP input_metric, SEXP categories, SEXP null_category, SEXP MO, SEXP TOA, SEXP TIA, SEXP T_categories, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(categories);
    PROTECT(null_category);
    PROTECT(MO);
    PROTECT(TOA);
    PROTECT(TIA);
    PROTECT(T_categories);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_categories = sexp_to_anyobjectptr(categories, T_categories);
    bool c_null_category = asLogical(null_category);
    char * c_MO = rt_to_string(MO);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_count_by_categories(c_input_domain, c_input_metric, c_categories, c_null_category, c_MO, c_TOA);

    UNPROTECT(9);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_count_distinct(
    SEXP input_domain, SEXP input_metric, SEXP TO, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(TO);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_TO = rt_to_string(TO);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_count_distinct(c_input_domain, c_input_metric, c_TO);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_create_dataframe(
    SEXP col_names, SEXP K, SEXP T_col_names, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(col_names);
    PROTECT(K);
    PROTECT(T_col_names);
    PROTECT(log);

    AnyObject * c_col_names = sexp_to_anyobjectptr(col_names, T_col_names);
    char * c_K = rt_to_string(K);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_create_dataframe(c_col_names, c_K);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_df_cast_default(
    SEXP input_domain, SEXP input_metric, SEXP column_name, SEXP TIA, SEXP TOA, SEXP TK, SEXP M, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(column_name);
    PROTECT(TIA);
    PROTECT(TOA);
    PROTECT(TK);
    PROTECT(M);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_column_name = sexp_to_anyobjectptr(column_name, TK);
    char * c_TIA = rt_to_string(TIA);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_df_cast_default(c_input_domain, c_input_metric, c_column_name, c_TIA, c_TOA);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_df_is_equal(
    SEXP input_domain, SEXP input_metric, SEXP column_name, SEXP value, SEXP TIA, SEXP TK, SEXP M, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(column_name);
    PROTECT(value);
    PROTECT(TIA);
    PROTECT(TK);
    PROTECT(M);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_column_name = sexp_to_anyobjectptr(column_name, TK);
    AnyObject * c_value = sexp_to_anyobjectptr(value, TIA);
    char * c_TIA = rt_to_string(TIA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_df_is_equal(c_input_domain, c_input_metric, c_column_name, c_value, c_TIA);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_drop_null(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_drop_null(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_find(
    SEXP input_domain, SEXP input_metric, SEXP categories, SEXP TIA, SEXP T_categories, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(categories);
    PROTECT(TIA);
    PROTECT(T_categories);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_categories = sexp_to_anyobjectptr(categories, T_categories);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_find(c_input_domain, c_input_metric, c_categories);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_find_bin(
    SEXP input_domain, SEXP input_metric, SEXP edges, SEXP TIA, SEXP T_edges, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(edges);
    PROTECT(TIA);
    PROTECT(T_edges);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_edges = sexp_to_anyobjectptr(edges, T_edges);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_find_bin(c_input_domain, c_input_metric, c_edges);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_identity(
    SEXP domain, SEXP metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(domain);
    PROTECT(metric);
    PROTECT(log);

    AnyDomain * c_domain = sexp_to_anydomainptr(domain);
    AnyMetric * c_metric = sexp_to_anymetricptr(metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_identity(c_domain, c_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_impute_constant(
    SEXP input_domain, SEXP input_metric, SEXP constant, SEXP T_constant, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(constant);
    PROTECT(T_constant);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_constant = sexp_to_anyobjectptr(constant, T_constant);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_impute_constant(c_input_domain, c_input_metric, c_constant);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_impute_uniform_float(
    SEXP input_domain, SEXP input_metric, SEXP bounds, SEXP TA, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(bounds);
    PROTECT(TA);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_impute_uniform_float(c_input_domain, c_input_metric, c_bounds);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_index(
    SEXP input_domain, SEXP input_metric, SEXP categories, SEXP null, SEXP TOA, SEXP T_categories, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(categories);
    PROTECT(null);
    PROTECT(TOA);
    PROTECT(T_categories);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_categories = sexp_to_anyobjectptr(categories, T_categories);
    AnyObject * c_null = sexp_to_anyobjectptr(null, TOA);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_index(c_input_domain, c_input_metric, c_categories, c_null, c_TOA);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_is_equal(
    SEXP input_domain, SEXP input_metric, SEXP value, SEXP TIA, SEXP M, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(value);
    PROTECT(TIA);
    PROTECT(M);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_value = sexp_to_anyobjectptr(value, TIA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_is_equal(c_input_domain, c_input_metric, c_value);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_is_null(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_is_null(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_lipschitz_float_mul(
    SEXP input_domain, SEXP input_metric, SEXP constant, SEXP bounds, SEXP TA, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(constant);
    PROTECT(bounds);
    PROTECT(TA);
    PROTECT(T_bounds);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    void * c_constant = sexp_to_voidptr(constant, TA);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_lipschitz_float_mul(c_input_domain, c_input_metric, c_constant, c_bounds);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_mean(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_mean(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_metric_bounded(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_metric_bounded(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_metric_unbounded(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_metric_unbounded(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_ordered_random(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_ordered_random(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_quantile_score_candidates(
    SEXP input_domain, SEXP input_metric, SEXP candidates, SEXP alpha, SEXP TIA, SEXP T_candidates, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(candidates);
    PROTECT(alpha);
    PROTECT(TIA);
    PROTECT(T_candidates);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    AnyObject * c_candidates = sexp_to_anyobjectptr(candidates, T_candidates);
    double c_alpha = Rf_asReal(alpha);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_quantile_score_candidates(c_input_domain, c_input_metric, c_candidates, c_alpha);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_quantiles_from_counts(
    SEXP bin_edges, SEXP alphas, SEXP interpolation, SEXP TA, SEXP F, SEXP T_bin_edges, SEXP T_alphas, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(bin_edges);
    PROTECT(alphas);
    PROTECT(interpolation);
    PROTECT(TA);
    PROTECT(F);
    PROTECT(T_bin_edges);
    PROTECT(T_alphas);
    PROTECT(log);

    AnyObject * c_bin_edges = sexp_to_anyobjectptr(bin_edges, T_bin_edges);
    AnyObject * c_alphas = sexp_to_anyobjectptr(alphas, T_alphas);
    char * c_interpolation = (char *)CHAR(STRING_ELT(interpolation, 0));
    char * c_TA = rt_to_string(TA);
    char * c_F = rt_to_string(F);

    // Call library function.
    FfiResult_____AnyFunction _result = opendp_transformations__make_quantiles_from_counts(c_bin_edges, c_alphas, c_interpolation, c_TA, c_F);

    UNPROTECT(8);
    if(_result.tag == Err_____AnyFunction)
        return(extract_error(_result.err));
    AnyFunction* _return_value = _result.ok;
    return(anyfunctionptr_to_sexp(_return_value, log));
}


SEXP transformations__make_resize(
    SEXP input_domain, SEXP input_metric, SEXP size, SEXP constant, SEXP MO, SEXP T_constant, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(size);
    PROTECT(constant);
    PROTECT(MO);
    PROTECT(T_constant);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_constant = sexp_to_anyobjectptr(constant, T_constant);
    char * c_MO = rt_to_string(MO);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_resize(c_input_domain, c_input_metric, c_size, c_constant, c_MO);

    UNPROTECT(7);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_select_column(
    SEXP key, SEXP K, SEXP TOA, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(key);
    PROTECT(K);
    PROTECT(TOA);
    PROTECT(log);

    AnyObject * c_key = sexp_to_anyobjectptr(key, K);
    char * c_K = rt_to_string(K);
    char * c_TOA = rt_to_string(TOA);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_select_column(c_key, c_K, c_TOA);

    UNPROTECT(4);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_float_checked_sum(
    SEXP size, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(S);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_float_checked_sum(c_size, c_bounds, c_S);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_float_ordered_sum(
    SEXP size, SEXP bounds, SEXP S, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(S);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_float_ordered_sum(c_size, c_bounds, c_S);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_int_checked_sum(
    SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_int_checked_sum(c_size, c_bounds, c_T);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_int_monotonic_sum(
    SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_int_monotonic_sum(c_size, c_bounds, c_T);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_int_ordered_sum(
    SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_int_ordered_sum(c_size, c_bounds, c_T);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sized_bounded_int_split_sum(
    SEXP size, SEXP bounds, SEXP T, SEXP T_bounds, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(size);
    PROTECT(bounds);
    PROTECT(T);
    PROTECT(T_bounds);
    PROTECT(log);

    size_t c_size = (size_t)Rf_asInteger(size);
    AnyObject * c_bounds = sexp_to_anyobjectptr(bounds, T_bounds);
    char * c_T = rt_to_string(T);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sized_bounded_int_split_sum(c_size, c_bounds, c_T);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_split_dataframe(
    SEXP separator, SEXP col_names, SEXP K, SEXP T_col_names, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(separator);
    PROTECT(col_names);
    PROTECT(K);
    PROTECT(T_col_names);
    PROTECT(log);

    char * c_separator = (char *)CHAR(STRING_ELT(separator, 0));
    AnyObject * c_col_names = sexp_to_anyobjectptr(col_names, T_col_names);
    char * c_K = rt_to_string(K);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_split_dataframe(c_separator, c_col_names, c_K);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_split_lines(
    SEXP log
) {
    // No arguments to convert to c types.
    PROTECT(log);
    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_split_lines();

    UNPROTECT(1);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_split_records(
    SEXP separator, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(separator);
    PROTECT(log);

    char * c_separator = (char *)CHAR(STRING_ELT(separator, 0));

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_split_records(c_separator);

    UNPROTECT(2);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_subset_by(
    SEXP indicator_column, SEXP keep_columns, SEXP TK, SEXP T_keep_columns, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(indicator_column);
    PROTECT(keep_columns);
    PROTECT(TK);
    PROTECT(T_keep_columns);
    PROTECT(log);

    AnyObject * c_indicator_column = sexp_to_anyobjectptr(indicator_column, TK);
    AnyObject * c_keep_columns = sexp_to_anyobjectptr(keep_columns, T_keep_columns);
    char * c_TK = rt_to_string(TK);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_subset_by(c_indicator_column, c_keep_columns, c_TK);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sum(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sum(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_sum_of_squared_deviations(
    SEXP input_domain, SEXP input_metric, SEXP S, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(S);
    PROTECT(T);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_sum_of_squared_deviations(c_input_domain, c_input_metric, c_S);

    UNPROTECT(5);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_unordered(
    SEXP input_domain, SEXP input_metric, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_unordered(c_input_domain, c_input_metric);

    UNPROTECT(3);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}


SEXP transformations__make_variance(
    SEXP input_domain, SEXP input_metric, SEXP ddof, SEXP S, SEXP T, SEXP log
) {
    // Convert arguments to c types.
    PROTECT(input_domain);
    PROTECT(input_metric);
    PROTECT(ddof);
    PROTECT(S);
    PROTECT(T);
    PROTECT(log);

    AnyDomain * c_input_domain = sexp_to_anydomainptr(input_domain);
    AnyMetric * c_input_metric = sexp_to_anymetricptr(input_metric);
    size_t c_ddof = (size_t)Rf_asInteger(ddof);
    char * c_S = rt_to_string(S);

    // Call library function.
    FfiResult_____AnyTransformation _result = opendp_transformations__make_variance(c_input_domain, c_input_metric, c_ddof, c_S);

    UNPROTECT(6);
    if(_result.tag == Err_____AnyTransformation)
        return(extract_error(_result.err));
    AnyTransformation* _return_value = _result.ok;
    return(anytransformationptr_to_sexp(_return_value, log));
}

