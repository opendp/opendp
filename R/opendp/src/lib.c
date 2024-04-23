// Auto-generated. Do not edit.
#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

// Import C headers for rust API
#include "Ropendp.h"

SEXP AnyObject_tag;
SEXP AnyTransformation_tag;
SEXP AnyMeasurement_tag;
SEXP AnyDomain_tag;
SEXP AnyMetric_tag;
SEXP AnyMeasure_tag;
SEXP AnyFunction_tag;

static R_CMethodDef R_CDef[] = {
    {"accuracy__accuracy_to_discrete_gaussian_scale", (DL_FUNC) &accuracy__accuracy_to_discrete_gaussian_scale, 4},
    {"accuracy__accuracy_to_discrete_laplacian_scale", (DL_FUNC) &accuracy__accuracy_to_discrete_laplacian_scale, 4},
    {"accuracy__accuracy_to_gaussian_scale", (DL_FUNC) &accuracy__accuracy_to_gaussian_scale, 4},
    {"accuracy__accuracy_to_laplacian_scale", (DL_FUNC) &accuracy__accuracy_to_laplacian_scale, 4},
    {"accuracy__discrete_gaussian_scale_to_accuracy", (DL_FUNC) &accuracy__discrete_gaussian_scale_to_accuracy, 4},
    {"accuracy__discrete_laplacian_scale_to_accuracy", (DL_FUNC) &accuracy__discrete_laplacian_scale_to_accuracy, 4},
    {"accuracy__gaussian_scale_to_accuracy", (DL_FUNC) &accuracy__gaussian_scale_to_accuracy, 4},
    {"accuracy__laplacian_scale_to_accuracy", (DL_FUNC) &accuracy__laplacian_scale_to_accuracy, 4},
    {"combinators__make_basic_composition", (DL_FUNC) &combinators__make_basic_composition, 3},
    {"combinators__make_chain_mt", (DL_FUNC) &combinators__make_chain_mt, 3},
    {"combinators__make_chain_pm", (DL_FUNC) &combinators__make_chain_pm, 3},
    {"combinators__make_chain_tt", (DL_FUNC) &combinators__make_chain_tt, 3},
    {"combinators__make_fix_delta", (DL_FUNC) &combinators__make_fix_delta, 4},
    {"combinators__make_population_amplification", (DL_FUNC) &combinators__make_population_amplification, 3},
    {"combinators__make_pureDP_to_fixed_approxDP", (DL_FUNC) &combinators__make_pureDP_to_fixed_approxDP, 2},
    {"combinators__make_pureDP_to_zCDP", (DL_FUNC) &combinators__make_pureDP_to_zCDP, 2},
    {"combinators__make_sequential_composition", (DL_FUNC) &combinators__make_sequential_composition, 9},
    {"combinators__make_zCDP_to_approxDP", (DL_FUNC) &combinators__make_zCDP_to_approxDP, 2},
    {"core__function_eval", (DL_FUNC) &core__function_eval, 5},
    {"core__measurement_check", (DL_FUNC) &core__measurement_check, 6},
    {"core__measurement_function", (DL_FUNC) &core__measurement_function, 2},
    {"core__measurement_input_carrier_type", (DL_FUNC) &core__measurement_input_carrier_type, 2},
    {"core__measurement_input_distance_type", (DL_FUNC) &core__measurement_input_distance_type, 2},
    {"core__measurement_input_domain", (DL_FUNC) &core__measurement_input_domain, 2},
    {"core__measurement_input_metric", (DL_FUNC) &core__measurement_input_metric, 2},
    {"core__measurement_invoke", (DL_FUNC) &core__measurement_invoke, 4},
    {"core__measurement_map", (DL_FUNC) &core__measurement_map, 4},
    {"core__measurement_output_distance_type", (DL_FUNC) &core__measurement_output_distance_type, 2},
    {"core__measurement_output_measure", (DL_FUNC) &core__measurement_output_measure, 2},
    {"core__queryable_eval", (DL_FUNC) &core__queryable_eval, 4},
    {"core__queryable_query_type", (DL_FUNC) &core__queryable_query_type, 2},
    {"core__transformation_check", (DL_FUNC) &core__transformation_check, 6},
    {"core__transformation_function", (DL_FUNC) &core__transformation_function, 2},
    {"core__transformation_input_carrier_type", (DL_FUNC) &core__transformation_input_carrier_type, 2},
    {"core__transformation_input_distance_type", (DL_FUNC) &core__transformation_input_distance_type, 2},
    {"core__transformation_input_domain", (DL_FUNC) &core__transformation_input_domain, 2},
    {"core__transformation_input_metric", (DL_FUNC) &core__transformation_input_metric, 2},
    {"core__transformation_invoke", (DL_FUNC) &core__transformation_invoke, 4},
    {"core__transformation_map", (DL_FUNC) &core__transformation_map, 4},
    {"core__transformation_output_distance_type", (DL_FUNC) &core__transformation_output_distance_type, 2},
    {"core__transformation_output_domain", (DL_FUNC) &core__transformation_output_domain, 2},
    {"core__transformation_output_metric", (DL_FUNC) &core__transformation_output_metric, 2},
    {"data__object_type", (DL_FUNC) &data__object_type, 2},
    {"data__smd_curve_epsilon", (DL_FUNC) &data__smd_curve_epsilon, 4},
    {"data__to_string", (DL_FUNC) &data__to_string, 2},
    {"domains__atom_domain", (DL_FUNC) &domains__atom_domain, 5},
    {"domains__domain_carrier_type", (DL_FUNC) &domains__domain_carrier_type, 2},
    {"domains__domain_debug", (DL_FUNC) &domains__domain_debug, 2},
    {"domains__domain_type", (DL_FUNC) &domains__domain_type, 2},
    {"domains__map_domain", (DL_FUNC) &domains__map_domain, 3},
    {"domains__member", (DL_FUNC) &domains__member, 4},
    {"domains__option_domain", (DL_FUNC) &domains__option_domain, 3},
    {"domains__vector_domain", (DL_FUNC) &domains__vector_domain, 4},
    {"measurements__make_alp_queryable", (DL_FUNC) &measurements__make_alp_queryable, 13},
    {"measurements__make_base_laplace_threshold", (DL_FUNC) &measurements__make_base_laplace_threshold, 7},
    {"measurements__make_gaussian", (DL_FUNC) &measurements__make_gaussian, 8},
    {"measurements__make_geometric", (DL_FUNC) &measurements__make_geometric, 8},
    {"measurements__make_laplace", (DL_FUNC) &measurements__make_laplace, 8},
    {"measurements__make_randomized_response", (DL_FUNC) &measurements__make_randomized_response, 7},
    {"measurements__make_randomized_response_bool", (DL_FUNC) &measurements__make_randomized_response_bool, 4},
    {"measurements__make_report_noisy_max_gumbel", (DL_FUNC) &measurements__make_report_noisy_max_gumbel, 6},
    {"measures__fixed_smoothed_max_divergence", (DL_FUNC) &measures__fixed_smoothed_max_divergence, 2},
    {"measures__max_divergence", (DL_FUNC) &measures__max_divergence, 2},
    {"measures__measure_debug", (DL_FUNC) &measures__measure_debug, 2},
    {"measures__measure_distance_type", (DL_FUNC) &measures__measure_distance_type, 2},
    {"measures__measure_type", (DL_FUNC) &measures__measure_type, 2},
    {"measures__smoothed_max_divergence", (DL_FUNC) &measures__smoothed_max_divergence, 2},
    {"measures__user_divergence", (DL_FUNC) &measures__user_divergence, 2},
    {"measures__zero_concentrated_divergence", (DL_FUNC) &measures__zero_concentrated_divergence, 2},
    {"metrics__absolute_distance", (DL_FUNC) &metrics__absolute_distance, 2},
    {"metrics__change_one_distance", (DL_FUNC) &metrics__change_one_distance, 1},
    {"metrics__discrete_distance", (DL_FUNC) &metrics__discrete_distance, 1},
    {"metrics__hamming_distance", (DL_FUNC) &metrics__hamming_distance, 1},
    {"metrics__insert_delete_distance", (DL_FUNC) &metrics__insert_delete_distance, 1},
    {"metrics__l1_distance", (DL_FUNC) &metrics__l1_distance, 2},
    {"metrics__l2_distance", (DL_FUNC) &metrics__l2_distance, 2},
    {"metrics__linf_distance", (DL_FUNC) &metrics__linf_distance, 3},
    {"metrics__metric_debug", (DL_FUNC) &metrics__metric_debug, 2},
    {"metrics__metric_distance_type", (DL_FUNC) &metrics__metric_distance_type, 2},
    {"metrics__metric_type", (DL_FUNC) &metrics__metric_type, 2},
    {"metrics__partition_distance", (DL_FUNC) &metrics__partition_distance, 2},
    {"metrics__symmetric_distance", (DL_FUNC) &metrics__symmetric_distance, 1},
    {"metrics__user_distance", (DL_FUNC) &metrics__user_distance, 2},
    {"transformations__choose_branching_factor", (DL_FUNC) &transformations__choose_branching_factor, 2},
    {"transformations__make_b_ary_tree", (DL_FUNC) &transformations__make_b_ary_tree, 5},
    {"transformations__make_bounded_float_checked_sum", (DL_FUNC) &transformations__make_bounded_float_checked_sum, 6},
    {"transformations__make_bounded_float_ordered_sum", (DL_FUNC) &transformations__make_bounded_float_ordered_sum, 6},
    {"transformations__make_bounded_int_monotonic_sum", (DL_FUNC) &transformations__make_bounded_int_monotonic_sum, 4},
    {"transformations__make_bounded_int_ordered_sum", (DL_FUNC) &transformations__make_bounded_int_ordered_sum, 4},
    {"transformations__make_bounded_int_split_sum", (DL_FUNC) &transformations__make_bounded_int_split_sum, 4},
    {"transformations__make_cast", (DL_FUNC) &transformations__make_cast, 4},
    {"transformations__make_cast_default", (DL_FUNC) &transformations__make_cast_default, 6},
    {"transformations__make_cast_inherent", (DL_FUNC) &transformations__make_cast_inherent, 4},
    {"transformations__make_cdf", (DL_FUNC) &transformations__make_cdf, 2},
    {"transformations__make_clamp", (DL_FUNC) &transformations__make_clamp, 6},
    {"transformations__make_consistent_b_ary_tree", (DL_FUNC) &transformations__make_consistent_b_ary_tree, 4},
    {"transformations__make_count", (DL_FUNC) &transformations__make_count, 4},
    {"transformations__make_count_by", (DL_FUNC) &transformations__make_count_by, 5},
    {"transformations__make_count_by_categories", (DL_FUNC) &transformations__make_count_by_categories, 9},
    {"transformations__make_count_distinct", (DL_FUNC) &transformations__make_count_distinct, 4},
    {"transformations__make_create_dataframe", (DL_FUNC) &transformations__make_create_dataframe, 4},
    {"transformations__make_df_cast_default", (DL_FUNC) &transformations__make_df_cast_default, 8},
    {"transformations__make_df_is_equal", (DL_FUNC) &transformations__make_df_is_equal, 8},
    {"transformations__make_drop_null", (DL_FUNC) &transformations__make_drop_null, 3},
    {"transformations__make_find", (DL_FUNC) &transformations__make_find, 6},
    {"transformations__make_find_bin", (DL_FUNC) &transformations__make_find_bin, 6},
    {"transformations__make_identity", (DL_FUNC) &transformations__make_identity, 3},
    {"transformations__make_impute_constant", (DL_FUNC) &transformations__make_impute_constant, 5},
    {"transformations__make_impute_uniform_float", (DL_FUNC) &transformations__make_impute_uniform_float, 6},
    {"transformations__make_index", (DL_FUNC) &transformations__make_index, 7},
    {"transformations__make_is_equal", (DL_FUNC) &transformations__make_is_equal, 6},
    {"transformations__make_is_null", (DL_FUNC) &transformations__make_is_null, 3},
    {"transformations__make_lipschitz_float_mul", (DL_FUNC) &transformations__make_lipschitz_float_mul, 7},
    {"transformations__make_mean", (DL_FUNC) &transformations__make_mean, 3},
    {"transformations__make_metric_bounded", (DL_FUNC) &transformations__make_metric_bounded, 3},
    {"transformations__make_metric_unbounded", (DL_FUNC) &transformations__make_metric_unbounded, 3},
    {"transformations__make_ordered_random", (DL_FUNC) &transformations__make_ordered_random, 3},
    {"transformations__make_quantile_score_candidates", (DL_FUNC) &transformations__make_quantile_score_candidates, 7},
    {"transformations__make_quantiles_from_counts", (DL_FUNC) &transformations__make_quantiles_from_counts, 8},
    {"transformations__make_resize", (DL_FUNC) &transformations__make_resize, 7},
    {"transformations__make_select_column", (DL_FUNC) &transformations__make_select_column, 4},
    {"transformations__make_sized_bounded_float_checked_sum", (DL_FUNC) &transformations__make_sized_bounded_float_checked_sum, 6},
    {"transformations__make_sized_bounded_float_ordered_sum", (DL_FUNC) &transformations__make_sized_bounded_float_ordered_sum, 6},
    {"transformations__make_sized_bounded_int_checked_sum", (DL_FUNC) &transformations__make_sized_bounded_int_checked_sum, 5},
    {"transformations__make_sized_bounded_int_monotonic_sum", (DL_FUNC) &transformations__make_sized_bounded_int_monotonic_sum, 5},
    {"transformations__make_sized_bounded_int_ordered_sum", (DL_FUNC) &transformations__make_sized_bounded_int_ordered_sum, 5},
    {"transformations__make_sized_bounded_int_split_sum", (DL_FUNC) &transformations__make_sized_bounded_int_split_sum, 5},
    {"transformations__make_split_dataframe", (DL_FUNC) &transformations__make_split_dataframe, 5},
    {"transformations__make_split_lines", (DL_FUNC) &transformations__make_split_lines, 1},
    {"transformations__make_split_records", (DL_FUNC) &transformations__make_split_records, 2},
    {"transformations__make_subset_by", (DL_FUNC) &transformations__make_subset_by, 5},
    {"transformations__make_sum", (DL_FUNC) &transformations__make_sum, 3},
    {"transformations__make_sum_of_squared_deviations", (DL_FUNC) &transformations__make_sum_of_squared_deviations, 5},
    {"transformations__make_unordered", (DL_FUNC) &transformations__make_unordered, 3},
    {"transformations__make_variance", (DL_FUNC) &transformations__make_variance, 6},
    {NULL, NULL, 0},
};

void R_init_opendp(DllInfo *dll)
{
    R_registerRoutines(dll, R_CDef, NULL, NULL, NULL);
    // here we create the tags for the external pointers
    AnyObject_tag = install("AnyObject_TAG");
    AnyTransformation_tag = install("AnyTransformation_TAG");
    AnyMeasurement_tag = install("AnyMeasurement_TAG");
    AnyDomain_tag = install("AnyDomain_TAG");
    AnyMetric_tag = install("AnyMetric_TAG");
    AnyMeasure_tag = install("AnyMeasure_TAG");
    AnyFunction_tag = install("AnyFunction_TAG");
    R_useDynamicSymbols(dll, TRUE);
}
