#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

#include "opendp.h"

// tag for external pointer objects
SEXP AnyObject_tag;
SEXP AnyMeasurement_tag;
SEXP AnyTransformation_tag;
SEXP SMDCurve_tag;
SEXP OPENDP_tag; 

SEXP charVec_tag;
SEXP char_tag;
SEXP double_tag;
SEXP int_tag;
SEXP bool_tag;


/* headers */
SEXP square_It(SEXP x);
SEXP apply_Fun(SEXP x, SEXP f, SEXP rho);
/* Two functions adapted from the code in man/doc/R-exts */

double feval(double x, SEXP f, SEXP rho);
SEXP mkans(double x);
  
// new function headers 
SEXP create(SEXP info);
SEXP get(SEXP XPtr);
SEXP set(SEXP XPtr, SEXP str);


// data
SEXP odp_slice_as_object(SEXP data);
SEXP odp_object_as_slice(SEXP obj);
SEXP odp_R_to_C(SEXP value, SEXP c_type, SEXP type_name);
struct AnyObject *SEXP2AnyObject(SEXP data);
struct AnyObject *SEXP2AnyObjectWithType(SEXP data, char *destType);
void *R2C(SEXP data);
SEXP extract_error(struct FfiError *err);
int is_measVec(SEXP x); 

// measurements
void odp_AnyMeasurement_finalizer(SEXP XPtr);
SEXP odp_measurement_invoke(SEXP meas, SEXP data);
SEXP odp_measurement_map(SEXP meas, SEXP data);
SEXP odp_make_base_discrete_laplace(SEXP scale, SEXP D, SEXP QO, SEXP info);
SEXP odp_make_base_gaussian(SEXP scale, SEXP k, SEXP D, SEXP MO, SEXP info);
SEXP odp_make_base_laplace(SEXP scale, SEXP k, SEXP D, SEXP info);
SEXP odp_make_randomized_response(SEXP categories, SEXP prob, SEXP constant_time, SEXP T, SEXP QO, SEXP info);
SEXP odp_make_base_ptr(SEXP scale, SEXP threshold, SEXP TK, SEXP k, SEXP TV, SEXP info);
SEXP odp_getMeasurementInfo(SEXP meas);

// transformations
SEXP returnAnyTransformation(FfiResult_____AnyTransformation result, SEXP info);
void odp_AnyTransformation_finalizer(SEXP XPtr);
SEXP odp_make_bounded_sum(SEXP bounds, SEXP MI, SEXP T, SEXP info);
SEXP odp_make_sized_bounded_mean(SEXP size, SEXP bounds, SEXP MI, SEXP T, SEXP info);
SEXP odp_make_clamp(SEXP bounds, SEXP TA, SEXP info);
SEXP odp_make_cast_default(SEXP TIA, SEXP TOA, SEXP info);
SEXP odp_make_count(SEXP TIA, SEXP TO, SEXP info);
SEXP odp_transformation_invoke(SEXP tran, SEXP data, SEXP info);
SEXP odp_transformation_map(SEXP tran, SEXP data);
SEXP odp_getTransformationInfo(SEXP tran);
SEXP odp_transformation_check(SEXP tran, SEXP distance_in, SEXP distance_out); 
SEXP odp_measurement_check(SEXP meas, SEXP distance_in, SEXP distance_out);
SEXP odp_smd_curve_epsilon(SEXP curve, SEXP delta);
SEXP odp_make_count_by_categories(SEXP categories, SEXP null_category, SEXP MO, SEXP TIA, SEXP TOA, SEXP info);
  
// combinations
SEXP odp_make_chain_mt(SEXP meas, SEXP tran, SEXP info);
SEXP odp_make_chain_tm(SEXP tran, SEXP meas, SEXP info);
SEXP odp_make_chain_tt(SEXP tran1, SEXP tran2, SEXP info);
SEXP odp_make_pureDP_to_fixed_approxDP(SEXP meas, SEXP info);
SEXP odp_make_zCDP_to_approxDP(SEXP meas, SEXP info);
SEXP odp_make_pureDP_to_zCDP(SEXP meas, SEXP info);
SEXP odp_make_fix_delta(SEXP meas, SEXP delta, SEXP info);
SEXP odp_make_basic_composition(SEXP measVec, SEXP info);
SEXP odp_make_population_amplification(SEXP meas, SEXP population_size, SEXP info);  
// internal function
SEXP int_object_as_slice(struct AnyObject *data);
  
// macros needed to cast R SEXPs to opendp.lib input arguments
#define CharPtr(x) (const char *)CHAR(STRING_ELT(x, 0)) // converts STRSXP (vector character string) to 'const char *'
#define Num2VoidPtr(x) (void *)&(REAL(x)[0]) // converts REALSXP (numeric) to 'void *'
#define Int2VoidPtr(x) (void *)&(INTEGER(x)[0]) // converts INTSXP (numeric) to 'void *'
#define Int2UInt(x) (unsigned int)(INTEGER(x)[0]) // converts INTSXP (numeric) to 'void *'
#define AsBool(x)   (bool)&(LOGICAL(x)[0]) // converts LOGSXP (numeric) to 'bool *'
#define AsInteger(x)   (int)(INTEGER(x)[0]) // converts LOGSXP (numeric) to 'bool *'



#define Check_char_Ptr(s) do {                            \
if (TYPEOF(s) != EXTPTRSXP ||                                  \
    R_ExternalPtrTag(s) !=  char_tag)                     \
  error("bad char object");                               \
} while (0)                                                    
  
#define Check_charVec_Ptr(s) do {         \
if (TYPEOF(s) != EXTPTRSXP ||               \
    R_ExternalPtrTag(s) !=  charVec_tag)  \
  error("bad charVec object");            \
} while (0)                                 
  

#define Check_int_Ptr(s) do {             \
  if (TYPEOF(s) != EXTPTRSXP ||               \
      R_ExternalPtrTag(s) !=  int_tag)     \
    error("bad int object");               \
} while (0)                                 

  
#define Check_double_Ptr(s) do {                 \
  if (TYPEOF(s) != EXTPTRSXP ||               \
      R_ExternalPtrTag(s) !=  double_tag)        \
    error("bad double object");                  \
} while (0)                                 


#define Check_AnyObject_Ptr(s) do {         \
if (TYPEOF(s) != EXTPTRSXP ||               \
    R_ExternalPtrTag(s) !=  AnyObject_tag)  \
  error("bad AnyObject object");            \
} while (0)                                 


#define Check_AnyMeasurement_Ptr(s) do {             \
if (TYPEOF(s) != EXTPTRSXP ||                        \
    R_ExternalPtrTag(s) !=  AnyMeasurement_tag)      \
  error("bad AnyMeasurement object");                \
} while (0)


#define Check_AnyTransformation_Ptr(s) do {             \
if (TYPEOF(s) != EXTPTRSXP ||                        \
    R_ExternalPtrTag(s) !=  AnyTransformation_tag)      \
  error("bad AnyTransformation object");                \
} while (0)

#define Check_SMDCurve_Ptr(s) do {             \
if (TYPEOF(s) != EXTPTRSXP ||                        \
    R_ExternalPtrTag(s) !=  SMDCurve_tag)      \
  error("bad SMDCurve object");                \
} while (0)

#define CHECK_OPENDP_OBJECT(s) do {             \
if (TYPEOF(s) != EXTPTRSXP ||                   \
    R_ExternalPtrTag(s) !=  OPENDP_tag)      \
  error("bad OPENDP object");                \
} while (0)                                     
  
