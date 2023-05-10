/* this file contains the Measurements calls */
/* data is here as well but we will move it */


#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

// #include "opendp_base.h"
// #include "opendp.h"

// Import C headers for rust API
#include "Ropendp.h"




SEXP processlist(SEXP lst);

/* dll entry points */
static R_CMethodDef R_CDef[] = {
  {"square_It", (DL_FUNC)&square_It, 1}, /* here you register the DLL entry points to local C functions and explicit the number of arguments */
  {"apply_Fun", (DL_FUNC)&apply_Fun, 3},
  {"create", (DL_FUNC)&create, 1},
  {"get", (DL_FUNC)&get, 1},
  {"set", (DL_FUNC)&set, 2},
  {"processlist", (DL_FUNC)&processlist, 1},
  {"odp_slice_as_object", (DL_FUNC) &odp_slice_as_object, 1},
  {"odp_object_as_slice", (DL_FUNC) &odp_object_as_slice, 1},
  {"odp_R_to_C", (DL_FUNC) &odp_R_to_C, 3},
  {"odp_make_base_discrete_laplace", (DL_FUNC) &odp_make_base_discrete_laplace, 4},
  {"odp_make_base_gaussian", (DL_FUNC) &odp_make_base_gaussian, 5},
  {"odp_make_base_laplace", (DL_FUNC) &odp_make_base_laplace, 4},
  {"odp_make_randomized_response", (DL_FUNC) &odp_make_randomized_response, 6},
  {"odp_make_base_ptr", (DL_FUNC) &odp_make_base_ptr, 6},
  {"odp_getMeasurementInfo", (DL_FUNC) &odp_getMeasurementInfo, 1},
  {"odp_measurement_invoke", (DL_FUNC) &odp_measurement_invoke, 2},
  {"odp_measurement_map", (DL_FUNC) &odp_measurement_map, 2},
  {"odp_measurement_check", (DL_FUNC) &odp_measurement_check, 3},
  {"odp_smd_curve_epsilon", (DL_FUNC) &odp_smd_curve_epsilon, 2},
  {"odp_transformation_invoke", (DL_FUNC) &odp_transformation_invoke, 2},
  {"odp_transformation_map", (DL_FUNC) &odp_transformation_map, 2},
  {"odp_make_bounded_sum", (DL_FUNC) &odp_make_bounded_sum, 4},
  {"odp_make_sized_bounded_mean", (DL_FUNC) &odp_make_sized_bounded_mean, 5},
  {"odp_make_clamp", (DL_FUNC) &odp_make_clamp, 3},
  {"odp_make_count", (DL_FUNC) &odp_make_count, 3},
  {"odp_make_cast_default", (DL_FUNC) &odp_make_cast_default, 3},
  {"odp_make_chain_mt", (DL_FUNC) &odp_make_chain_mt, 3},
  {"odp_make_chain_tm", (DL_FUNC) &odp_make_chain_tm, 3},
  {"odp_make_chain_tt", (DL_FUNC) &odp_make_chain_tt, 3},
  {"odp_make_pureDP_to_fixed_approxDP", (DL_FUNC) &odp_make_pureDP_to_fixed_approxDP, 2},
  {"odp_make_zCDP_to_approxDP", (DL_FUNC) &odp_make_zCDP_to_approxDP, 2},
  {"odp_make_pureDP_to_zCDP", (DL_FUNC) &odp_make_pureDP_to_zCDP, 2},
  {"odp_make_fix_delta", (DL_FUNC) &odp_make_fix_delta, 3},
  {"odp_make_basic_composition", (DL_FUNC) &odp_make_basic_composition, 2},
  {"odp_make_population_amplification", (DL_FUNC) &odp_make_population_amplification, 3},
  {"odp_getTransformationInfo", (DL_FUNC) &odp_getTransformationInfo, 1},
  {"odp_transformation_check", (DL_FUNC) &odp_transformation_check, 3},
  {"odp_make_count_by_categories", (DL_FUNC) &odp_transformation_check, 6},
  {NULL, NULL, 0},
};


void R_init_opendp(DllInfo *dll)
{
  R_registerRoutines(dll, R_CDef, NULL, NULL, NULL);
// here we create the tags for the external pointers
  AnyObject_tag = install("AnyObject_TAG");
  AnyMeasurement_tag =  install("AnyMeasurement_TAG");
  SMDCurve_tag =  install("SMDCurve_TAG");
  AnyTransformation_tag =  install("AnyTransformation_TAG");
  OPENDP_tag = install("OPENDP_TAG");
  charVec_tag = install("charVec_tag");
  char_tag = install("char_tag");
  double_tag = install("double_tag");
  int_tag = install("int_tag");
  bool_tag = install("bool_tag");
  
  R_useDynamicSymbols(dll, TRUE);
}


 
