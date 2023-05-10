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



void odp_AnyTransformation_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_AnyTransformation_Ptr(XPtr); 
 // Rprintf("\n*** finalizing AnyTransformation ***\n\n");
  struct AnyTransformation *ptr = (struct AnyTransformation *) R_ExternalPtrAddr(XPtr);
  opendp_core___transformation_free(ptr);
  R_ClearExternalPtr(XPtr);
}



// struct FfiResult_____AnyTransformation opendp_transformations__make_count_by_categories(const struct AnyObject *categories,
//                                                                                           c_bool null_category,
//                                                                                           const char *MO,
//                                                                                           const char *TI,
//                                                                                           const char *TO);

SEXP odp_make_count_by_categories(SEXP categories, SEXP null_category, SEXP MO, SEXP TIA, SEXP TOA, SEXP info){
  if (TYPEOF(TIA) != STRSXP) {
    error("'TIA' must be a string");
  }
  if (TYPEOF(TOA) != STRSXP) {
    error("'TOA' must be a string");
  }
  if (TYPEOF(MO) != STRSXP) {
    error("'MO' must be a string");
  }
  if (TYPEOF(null_category) != LGLSXP) {
    error("'null_category' must be logical");
  }
  PROTECT(TIA = AS_CHARACTER(TIA));
  PROTECT(TOA = AS_CHARACTER(TOA));
  PROTECT(MO = AS_CHARACTER(MO));
  PROTECT(categories);
  PROTECT(null_category = AS_LOGICAL(null_category));
  
  
  UNPROTECT(5);
  
  FfiResult_____AnyTransformation result = opendp_transformations__make_count_by_categories(SEXP2AnyObject(categories),
                                                                                           AsBool(null_category),
                                                                                           CharPtr(MO),
                                                                                           CharPtr(TIA),
                                                                                           CharPtr(TOA));
  
  return( returnAnyTransformation(result, info) );
}
  
// struct FfiResult_____AnyTransformation opendp_transformations__make_cast_default(const char *TIA,
//                                                                                  const char *TOA);

SEXP odp_make_cast_default(SEXP TIA, SEXP TOA, SEXP info){
  if (TYPEOF(TIA) != STRSXP) {
    error("'TIA' must be a string");
  }
  if (TYPEOF(TOA) != STRSXP) {
    error("'TOA' must be a string");
  }
  
  PROTECT(TIA = AS_CHARACTER(TIA));
  PROTECT(TOA = AS_CHARACTER(TOA));
  
  FfiResult_____AnyTransformation result = opendp_transformations__make_cast_default(CharPtr(TIA),
                                                                              CharPtr(TOA));
  
  UNPROTECT(2);
  return( returnAnyTransformation(result, info) );
}

// struct FfiResult_____AnyTransformation opendp_transformations__make_clamp(const struct AnyObject *bounds,
//                                                                           const char *TA);

  
SEXP odp_make_clamp(SEXP bounds, SEXP TA, SEXP info){

    if (TYPEOF(TA) != STRSXP) {
      error("'TA' must be a string");
    }

    PROTECT(TA = AS_CHARACTER(TA));
    PROTECT(bounds);
    if( ((TYPEOF(bounds) != VECSXP) & (TYPEOF(bounds) != LISTSXP)) | (length(bounds)<2)) {
      UNPROTECT(2);
      error("'bounds must be a tuple");
    }
    FfiResult_____AnyTransformation result = opendp_transformations__make_clamp(SEXP2AnyObject(bounds),
                                                                              CharPtr(TA));
    UNPROTECT(2);
    
    return( returnAnyTransformation(result, info) );
}

SEXP returnAnyTransformation(FfiResult_____AnyTransformation result, SEXP info){
  if(result.tag == Ok_____AnyTransformation){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyTransformation_tag, info));
    R_RegisterCFinalizerEx(XPtr, odp_AnyTransformation_finalizer,TRUE);
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);
    
    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("TransformationPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  return(extract_error(result.err));
}

// struct FfiResult_____AnyTransformation opendp_transformations__make_sized_bounded_mean(unsigned int size,
//                                                                                        const struct AnyObject *bounds,
//                                                                                        const char *MI,
//                                                                                        const char *T);
SEXP odp_make_sized_bounded_mean(SEXP size, SEXP bounds, SEXP MI, SEXP T, SEXP info){
  
  if (TYPEOF(size) != INTSXP) {
    error("'size' must be integer");
  }

  if (TYPEOF(MI) != STRSXP) {
    error("'MI' must be a string");
  }
  
  if (TYPEOF(T) != STRSXP) {
    error("'T' must be a string");
  }
  
  PROTECT(size = AS_INTEGER(size));
  PROTECT(MI = AS_CHARACTER(MI));
  PROTECT(T = AS_CHARACTER(T));
  PROTECT(bounds);
  if( ((TYPEOF(bounds) != VECSXP) & (TYPEOF(bounds) != LISTSXP)) | (length(bounds)<2)) {
    UNPROTECT(4);
    error("'bounds must be a tuple");
  }
  FfiResult_____AnyTransformation result = opendp_transformations__make_sized_bounded_mean(Int2UInt(size),
                                                                                    SEXP2AnyObject(bounds),
                                                                                    CharPtr(MI),
                                                                                    CharPtr(T));
  UNPROTECT(4);  
  
  if(result.tag == Ok_____AnyTransformation){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyTransformation_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyTransformation_finalizer,TRUE); 
    UNPROTECT(1);
    
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);
    
    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("TransformationPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  return(extract_error(result.err));
  
}

// struct FfiResult_____AnyTransformation opendp_transformations__make_count(const char *TIA,
//                                                                           const char *TO);

SEXP odp_make_count(SEXP TIA, SEXP TO, SEXP info){
  if (TYPEOF(TIA) != STRSXP) {
    error("'TIA' must be a string");
  }
  if (TYPEOF(TO) != STRSXP) {
    error("'TO' must be a string");
  }
  
  PROTECT(TIA = AS_CHARACTER(TIA));
  PROTECT(TO = AS_CHARACTER(TO));

  FfiResult_____AnyTransformation result = opendp_transformations__make_count(CharPtr(TIA),
                                                                              CharPtr(TO));
  UNPROTECT(2);  
  
  if(result.tag == Ok_____AnyTransformation){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyTransformation_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyTransformation_finalizer,TRUE); 
    UNPROTECT(1);
    
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);
    
    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("TransformationPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  return(extract_error(result.err));
}


// bounds is tuple, i.e. and R list or generic vector
SEXP odp_make_bounded_sum(SEXP bounds, SEXP MI, SEXP T, SEXP info){
  
  if (TYPEOF(MI) != STRSXP) {
    error("'MI' must be a string");
  }
  
  if (TYPEOF(T) != STRSXP) {
    error("'T' must be a string");
  }
  
  PROTECT(MI = AS_CHARACTER(MI));
  PROTECT(T = AS_CHARACTER(T));
  PROTECT(bounds);
  if( ((TYPEOF(bounds) != VECSXP) & (TYPEOF(bounds) != LISTSXP)) | (length(bounds)<2)) {
    UNPROTECT(3);
    error("'bounds must be a tuple");
  }
  FfiResult_____AnyTransformation result = opendp_transformations__make_bounded_sum(SEXP2AnyObject(bounds),
                                                                                    CharPtr(MI),
                                                                                    CharPtr(T));
  UNPROTECT(3);  
 
  if(result.tag == Ok_____AnyTransformation){
    
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyTransformation_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyTransformation_finalizer,TRUE); 
    UNPROTECT(1);
    
    
    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("TransformationPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  return(extract_error(result.err));

}



// struct FfiResult_____AnyObject opendp_core__transformation_invoke(const AnyTransformation *this_,
//                                                                   const struct AnyObject *arg);

SEXP odp_transformation_invoke(SEXP tran, SEXP data, SEXP info) {
  
  Check_AnyTransformation_Ptr(tran); 
  
  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  
  AnyObject *data_ptr = SEXP2AnyObject(data);
  
  FfiResult_____AnyObject result = opendp_core__transformation_invoke(tran_ptr, data_ptr);
  
  // printf("Success or error (transformation invoke): %d\n", result.tag);
  
  if(result.tag != Ok_____AnyObject)
    return(extract_error(result.err));
  
  return( int_object_as_slice(result.ok) );
  
}



SEXP odp_transformation_map(SEXP tran, SEXP data) {
  char *typename = Calloc(9, char);
  
  Check_AnyTransformation_Ptr(tran); 
  
  PROTECT(tran);
  PROTECT(data);
  
  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  struct FfiResult_____c_char input_distance = opendp_core__transformation_input_distance_type(tran_ptr);
  
  AnyObject *data_ptr = SEXP2AnyObjectWithType(data, input_distance.ok);
  
  struct FfiResult_____c_char data_expected = opendp_core__transformation_input_distance_type(tran_ptr);
  struct FfiResult_____c_char data_type = opendp_data__object_type(data_ptr);
  
  UNPROTECT(2);
  // the next code captures the u32 vs i32 case, we should probably do it when
  // we build the objects in SEXP2AnyObject though we do not have the 
  // unsigned int type in R
  
  if((data_expected.tag == Ok_____c_char) & (data_type.tag == Ok_____c_char)){
   // printf("\ndata type=%s, data type=%s\n", data_expected.ok, data_type.ok);
    if(strcmp(data_expected.ok,"u32")==0){
      strcpy(typename, "u32");
      FfiResult_____FfiSlice val = opendp_data__object_as_slice(data_ptr);
      if(val.tag !=  Ok_____FfiSlice)
        extract_error(val.err);
      int len = (int)val.ok->len;
      
     // printf("\n len =%d",len);
      if(len==1){
        unsigned int newdata = (unsigned int)( *(int *)(val.ok->ptr) );
       // printf("\ndata = %d, newdata=%u\n", *(int *)(val.ok->ptr), newdata);
        FfiSlice slice = {&newdata, len};
        FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);
        Free(typename);
        if(result.tag != Ok_____AnyObject){
          extract_error(result.err);
        }
        data_ptr = result.ok;
      } else {
        error("data type should ne of length 1");
      }
    }
  }
  

  FfiResult_____AnyObject result = opendp_core__transformation_map(tran_ptr, data_ptr);
  
  //printf("Success or error (transformation map): %d\n", result.tag);
  
  if(result.tag != Ok_____AnyObject)
    return(extract_error(result.err));
  
  return( int_object_as_slice(result.ok) );
  
}


// struct FfiResult_____c_bool opendp_core__transformation_check(const AnyTransformation *transformation,
//                                                               const struct AnyObject *distance_in,
//                                                               const struct AnyObject *distance_out);


SEXP odp_transformation_check(SEXP tran, SEXP distance_in, SEXP distance_out){ 
  Check_AnyTransformation_Ptr(tran); 
  PROTECT(tran);
  PROTECT(distance_in);
  PROTECT(distance_out);
  
  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  
  struct FfiResult_____c_char input_distance = opendp_core__transformation_input_distance_type(tran_ptr);
  struct FfiResult_____c_char output_distance = opendp_core__transformation_output_distance_type(tran_ptr);
  
  
  struct FfiResult_____c_bool result = opendp_core__transformation_check(tran_ptr,
                                                                         SEXP2AnyObjectWithType(distance_in,input_distance.ok),
                                                                         SEXP2AnyObjectWithType(distance_out,output_distance.ok));
  UNPROTECT(3);
  if(result.tag != Ok_____c_bool){
    return(extract_error(result.err));
  }
  
  SEXP okresult = PROTECT(Rf_ScalarLogical(*(bool *)result.ok));
  UNPROTECT(1);
  return(okresult);
  
  
}

SEXP odp_getTransformationInfo(SEXP tran){ 
  Check_AnyTransformation_Ptr(tran); 
  PROTECT(tran);
  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  struct FfiResult_____c_char input_carrier = opendp_core__transformation_input_carrier_type(tran_ptr);
  struct FfiResult_____c_char input_distance = opendp_core__transformation_input_distance_type(tran_ptr);
  struct FfiResult_____c_char output_distance = opendp_core__transformation_output_distance_type(tran_ptr);
  
  SEXP attr = PROTECT(allocVector(VECSXP, 4));
  SET_VECTOR_ELT(attr, 0, mkString(input_carrier.ok));
  SET_VECTOR_ELT(attr, 1, mkString(input_distance.ok));
  SET_VECTOR_ELT(attr, 2, mkString(output_distance.ok));
  SET_VECTOR_ELT(attr, 3, R_ExternalPtrProtected(tran));
  
  SEXP attrnames = PROTECT(allocVector(STRSXP, 4));
  SET_STRING_ELT(attrnames, 0, mkChar("input_carrier"));
  SET_STRING_ELT(attrnames, 1, mkChar("input_distance"));
  SET_STRING_ELT(attrnames, 2, mkChar("output_distance"));
  SET_STRING_ELT(attrnames, 3, mkChar("parameters"));
  setAttrib(attr, R_NamesSymbol, attrnames);
  UNPROTECT(3);
  return(attr);
}