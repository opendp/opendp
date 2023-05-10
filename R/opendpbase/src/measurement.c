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



void odp_AnyMeasurement_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_AnyMeasurement_Ptr(XPtr); 
 // Rprintf("\n*** finalizing AnyMeasurement ***\n\n");
  struct AnyMeasurement *ptr = (struct AnyMeasurement *) R_ExternalPtrAddr(XPtr);
  opendp_core___measurement_free(ptr);
  R_ClearExternalPtr(XPtr);
}


// struct FfiResult_____c_bool opendp_core__measurement_check(const AnyMeasurement *measurement,
//                                                            const struct AnyObject *distance_in,
//                                                            const struct AnyObject *distance_out);


SEXP odp_measurement_check(SEXP meas, SEXP distance_in, SEXP distance_out){ 
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  PROTECT(distance_in);
  PROTECT(distance_out);
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);

  struct FfiResult_____c_char input_distance = opendp_core__measurement_input_distance_type(meas_ptr);
  struct FfiResult_____c_char output_distance = opendp_core__measurement_output_distance_type(meas_ptr);
  

  FfiResult_____c_bool result = opendp_core__measurement_check(meas_ptr,
                                                              SEXP2AnyObjectWithType(distance_in, input_distance.ok),
                                                              SEXP2AnyObjectWithType(distance_out,output_distance.ok));
  UNPROTECT(3);
  if(result.tag != Ok_____c_bool){
    return(extract_error(result.err));
  }
  
  SEXP okresult = PROTECT(Rf_ScalarLogical(*(bool *)result.ok));
  UNPROTECT(1);
  return(okresult);
  
}

// struct FfiResult_____AnyObject opendp_data__smd_curve_epsilon(const struct AnyObject *curve,
//                                                               const struct AnyObject *delta);

SEXP odp_smd_curve_epsilon(SEXP curve, SEXP delta){
  Check_SMDCurve_Ptr(curve); 
  PROTECT(curve);
  PROTECT(delta);

  AnyMeasurement *curve_ptr = (AnyMeasurement *) R_ExternalPtrAddr(curve);
  
  FfiResult_____AnyObject result = opendp_data__smd_curve_epsilon(curve_ptr,
                                                               SEXP2AnyObject(delta));
  UNPROTECT(2);
  
  if(result.tag != Ok_____AnyObject)
    return(extract_error(result.err));
  
  return( int_object_as_slice(result.ok) );
  
}


// struct FfiResult_____AnyMeasurement opendp_measurements__make_base_ptr(const void *scale,
//                                                                        const void *threshold,
//                                                                        long k,
//                                                                        const char *TK,
//                                                                        const char *TV);

SEXP odp_make_base_ptr(SEXP scale, SEXP threshold, SEXP TK, SEXP k, SEXP TV, SEXP info){
  
  if ((TYPEOF(scale) != REALSXP) | (LENGTH(scale) > 1)) {
    error("'Input 'scale' must be a scalar numeric");
  }
  
  if ((TYPEOF(threshold) != REALSXP) | (LENGTH(threshold) > 1)) {
    error("'Input 'threshold' must be a scalar numeric");
  }
  
  if (TYPEOF(TK) != STRSXP) {
    error("'TK' must be a string");
  }
  
  if((TYPEOF(k) != INTSXP) | (LENGTH(k) != 1)) {
    error("'Input 'k' must be an integer scalar");
  }
  
  if (TYPEOF(TV) != STRSXP) {
    error("'TV' must be a string");
  }

  
  
  PROTECT(scale = AS_NUMERIC(scale));
  PROTECT(threshold = AS_NUMERIC(threshold));
  PROTECT(TK= AS_CHARACTER(TK));
  PROTECT(k = AS_INTEGER(k));
  PROTECT(TV = AS_CHARACTER(TV));
  
  FfiResult_____AnyMeasurement result = opendp_measurements__make_base_ptr(Num2VoidPtr(scale),
                                                                           Num2VoidPtr(threshold),
                                                                           AsInteger(k),
                                                                           CharPtr(TK),
                                                                           CharPtr(TV));
  
  UNPROTECT(5);
  if(result.tag == Ok_____AnyMeasurement){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); 
    UNPROTECT(1);
   
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);
    
    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  
  return(extract_error(result.err));
  

}



// opendp_measurements__make_randomized_response(const struct AnyObject *categories,
//                                                                                  const void *prob,
//                                                                                  c_bool constant_time,
//                                                                                  const char *T,
//                                                                                  const char *QO);


SEXP odp_make_randomized_response(SEXP categories, SEXP prob, SEXP constant_time, SEXP T, SEXP QO, SEXP info){
  
  if (TYPEOF(categories) != STRSXP) {
    error("'categories' must be a string vector");
  }
  
  if ((TYPEOF(prob) != REALSXP) | (LENGTH(prob) > 1)) {
    error("'Input 'prob' must be a scalar numeric");
  }
  
  if (TYPEOF(constant_time) != LGLSXP) {
    error("'constant_time' must be boolean");
  }
  
  if (TYPEOF(QO) != STRSXP) {
    error("'MO' must be a string");
  }
  
  if (TYPEOF(T) != STRSXP) {
    error("'T' must be a string");
  }
  
  PROTECT(prob = AS_NUMERIC(prob));
  PROTECT(T = AS_CHARACTER(T));
  PROTECT(constant_time = AS_LOGICAL(constant_time));
  PROTECT(QO = AS_CHARACTER(QO));
  PROTECT(categories);
  
  FfiResult_____AnyMeasurement result = opendp_measurements__make_randomized_response(SEXP2AnyObject(categories),
                                                                                      Num2VoidPtr(prob),
                                                                                      AsBool(constant_time),
                                                                                      CharPtr(T),
                                                                                      CharPtr(QO));
  UNPROTECT(5);
  
  if(result.tag == Ok_____AnyMeasurement){
    
   // printf("ok");
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); 
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
// printf("no");
  
 
 return(extract_error(result.err));
  
}


// struct FfiResult_____AnyMeasurement opendp_measurements__make_base_laplace(const void *scale,
//                                                                            long k,
//                                                                            const char *D);
SEXP odp_make_base_laplace(SEXP scale, SEXP k, SEXP D, SEXP info){
  
  if((TYPEOF(scale) != REALSXP) | (LENGTH(scale) != 1)) {
    error("'Input 'scale' must be a numeric scalar");
  }
  
  if((TYPEOF(k) != INTSXP) | (LENGTH(k) != 1)) {
    error("'Input 'k' must be an integer scalar");
  }
  
  if (TYPEOF(D) != STRSXP) {
    error("'D' must be a string");
  }
  
  PROTECT(scale = AS_NUMERIC(scale));
  PROTECT(k = AS_INTEGER(k));
  PROTECT(D = AS_CHARACTER(D));
  
  FfiResult_____AnyMeasurement result = opendp_measurements__make_base_laplace(Num2VoidPtr(scale),
                                                                               AsInteger(k),
                                                                               CharPtr(D));
  UNPROTECT(3);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); // we protect it in the LIST below
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); // important to register the proper finalizer
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  

  return(extract_error(result.err));
  
}

SEXP odp_make_base_discrete_laplace(SEXP scale, SEXP D, SEXP QO, SEXP info){
  
  if (TYPEOF(D) != STRSXP) {
    error("'D' must be a character vector");
  }
  
  if (TYPEOF(QO) != STRSXP) {
    error("'QO' must be a character vector");
  }
  
  if (LENGTH(scale) != 1) {
    error("'Input 'scale' must be a scalar");
  }
  
  if (TYPEOF(scale) != REALSXP) {
    error("'Input 'scale' must be numeric");
  }
  
  PROTECT(scale = AS_NUMERIC(scale));
  PROTECT(D = AS_CHARACTER(D));
  PROTECT(QO = AS_CHARACTER(QO));
  
  // struct FfiResult_____AnyMeasurement opendp_measurements__make_base_discrete_laplace(const void *scale,
  //                                                                                     const char *D,
  //                                                                                     const char *QO);
  // 
  FfiResult_____AnyMeasurement result = opendp_measurements__make_base_discrete_laplace(Num2VoidPtr(scale),CharPtr(D),CharPtr(QO));
  
  UNPROTECT(3);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); // we protect it in the LIST below
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); // important to register the proper finalizer
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
   
   return(extract_error(result.err));
  
}

// struct FfiResult_____AnyMeasurement opendp_measurements__make_base_gaussian(const void *scale,
//                                                                             long k,
//                                                                             const char *D,
//                                                                             const char *MO);

SEXP odp_make_base_gaussian(SEXP scale, SEXP k, SEXP D, SEXP MO, SEXP info){
  if (TYPEOF(D) != STRSXP) {
    error("'D' must be a string");
  }
  
  if (TYPEOF(MO) != STRSXP) {
    error("'MO' must be a string");
  }
  
  if( (TYPEOF(k) != INTSXP) | (LENGTH(k) != 1) ){
    error("'Input 'k' must be a scalar integer");
  }

  if( (LENGTH(scale) != 1) | (TYPEOF(scale) != REALSXP)){
    error("'Input 'scale' must be a scalar numeric");
  }

    
  PROTECT(scale = AS_NUMERIC(scale));
  PROTECT(k = AS_INTEGER(k));
  PROTECT(D = AS_CHARACTER(D));
  PROTECT(MO = AS_CHARACTER(MO));

  FfiResult_____AnyMeasurement result = opendp_measurements__make_base_gaussian(Num2VoidPtr(scale), 
                                                                                AsInteger(k),
                                                                                CharPtr(D), 
                                                                                CharPtr(MO));
  UNPROTECT(4);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); 
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  

  return(extract_error(result.err));
}




SEXP odp_measurement_invoke(SEXP meas, SEXP data) {

  Check_AnyMeasurement_Ptr(meas); 
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  struct FfiResult_____c_char input_carrier = opendp_core__measurement_input_carrier_type(meas_ptr);
  
  AnyObject *data_ptr = SEXP2AnyObjectWithType(data, input_carrier.ok);

  FfiResult_____AnyObject result = opendp_core__measurement_invoke(meas_ptr, data_ptr);

  //printf("Success or error (measure invoke): %d\n", result.tag);
   
  if(result.tag == Ok_____AnyObject){
    return( int_object_as_slice(result.ok) );
  }
  
  return(extract_error(result.err));
  

}



SEXP odp_measurement_map(SEXP meas, SEXP data) {
  
  Check_AnyMeasurement_Ptr(meas); 
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  struct FfiResult_____c_char input_distance = opendp_core__measurement_input_distance_type(meas_ptr);
  
  AnyObject *data_ptr = SEXP2AnyObjectWithType(data, input_distance.ok);
  
  FfiResult_____AnyObject result = opendp_core__measurement_map(meas_ptr, data_ptr);
  
  //printf("Success or error (measure map): %d\n", result.tag);
  
  if(result.tag == Ok_____AnyObject){
    return( int_object_as_slice(result.ok) );
  }
  
  return(extract_error(result.err));
  
  
}

SEXP odp_getMeasurementInfo(SEXP meas){ 
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  struct FfiResult_____c_char input_carrier = opendp_core__measurement_input_carrier_type(meas_ptr);
  struct FfiResult_____c_char input_distance = opendp_core__measurement_input_distance_type(meas_ptr);
  struct FfiResult_____c_char output_distance = opendp_core__measurement_output_distance_type(meas_ptr);
  
  SEXP attr = PROTECT(allocVector(VECSXP, 4));
  SET_VECTOR_ELT(attr, 0, mkString(input_carrier.ok));
  SET_VECTOR_ELT(attr, 1, mkString(input_distance.ok));
  SET_VECTOR_ELT(attr, 2, mkString(output_distance.ok));
  SET_VECTOR_ELT(attr, 3, R_ExternalPtrProtected(meas));
  
  SEXP attrnames = PROTECT(allocVector(STRSXP, 4));
  SET_STRING_ELT(attrnames, 0, mkChar("input_carrier"));
  SET_STRING_ELT(attrnames, 1, mkChar("input_distance"));
  SET_STRING_ELT(attrnames, 2, mkChar("output_distance"));
  SET_STRING_ELT(attrnames, 3, mkChar("parameters"));
  setAttrib(attr, R_NamesSymbol, attrnames);
  
  UNPROTECT(3);
  
  return(attr);
}
