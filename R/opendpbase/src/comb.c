/* this file contains the comb.c */


#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>

#include "Ropendp.h"


// struct FfiResult_____AnyMeasurement opendp_combinators__make_population_amplification(const AnyMeasurement *measurement,
//                                                                                       unsigned int population_size);


SEXP odp_make_population_amplification(SEXP meas, SEXP population_size, SEXP info){
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  PROTECT(population_size = AS_INTEGER(population_size));
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  
  struct FfiResult_____AnyMeasurement result = opendp_combinators__make_population_amplification(meas_ptr,
                                                                                                 Int2UInt(population_size));
  UNPROTECT(2);
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
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


// struct FfiResult_____AnyMeasurement opendp_combinators__make_basic_composition(const struct AnyObject *measurements);

SEXP odp_make_basic_composition(SEXP measVec, SEXP info){
  
  PROTECT(measVec);

  FfiResult_____AnyMeasurement result = opendp_combinators__make_basic_composition(SEXP2AnyObject(measVec));
  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
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

//struct FfiResult_____AnyMeasurement opendp_combinators__make_fix_delta(const AnyMeasurement *measurement,
//                                                                       const struct AnyObject *delta);
SEXP odp_make_fix_delta(SEXP meas, SEXP delta, SEXP info){
    Check_AnyMeasurement_Ptr(meas); 
    if((TYPEOF(delta) != REALSXP) | (LENGTH(delta) != 1)) {
      error("'Input 'delta' must be a numeric scalar");
    }
    PROTECT(meas);
    PROTECT(delta = AS_NUMERIC(delta));

    AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
    
    FfiResult_____AnyMeasurement result = opendp_combinators__make_fix_delta(meas_ptr, SEXP2AnyObject(delta));
    
    UNPROTECT(2);
    
    if(result.tag == Ok_____AnyMeasurement){
      
      //  printf("ok");
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



//struct FfiResult_____AnyMeasurement opendp_combinators__make_pureDP_to_zCDP(const AnyMeasurement *measurement);

SEXP odp_make_pureDP_to_zCDP(SEXP meas, SEXP info){
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  
  FfiResult_____AnyMeasurement result = opendp_combinators__make_pureDP_to_zCDP(meas_ptr);
  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
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

//struct FfiResult_____AnyMeasurement opendp_combinators__make_zCDP_to_approxDP(const AnyMeasurement *measurement);
SEXP odp_make_zCDP_to_approxDP(SEXP meas, SEXP info){
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  
  FfiResult_____AnyMeasurement result = opendp_combinators__make_zCDP_to_approxDP(meas_ptr);
  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
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
  
  
//struct FfiResult_____AnyMeasurement opendp_combinators__make_pureDP_to_fixed_approxDP(const AnyMeasurement *measurement);

SEXP odp_make_pureDP_to_fixed_approxDP(SEXP meas, SEXP info){
  Check_AnyMeasurement_Ptr(meas); 
  PROTECT(meas);
  
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  
  FfiResult_____AnyMeasurement result = opendp_combinators__make_pureDP_to_fixed_approxDP(meas_ptr);
  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
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


//struct FfiResult_____AnyMeasurement opendp_combinators__make_chain_mt(const AnyMeasurement *measurement1,
//            const AnyTransformation *transformation0);

// bounds is tuple, i.e. and R list
SEXP odp_make_chain_mt(SEXP meas, SEXP tran, SEXP info){
  
  
  Check_AnyMeasurement_Ptr(meas); 
  Check_AnyTransformation_Ptr(tran); 

  PROTECT(meas);
  PROTECT(tran);

  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
    
  FfiResult_____AnyMeasurement result = opendp_combinators__make_chain_mt(meas_ptr,tran_ptr);
  
  UNPROTECT(2);
  
  if(result.tag == Ok_____AnyMeasurement){
    
  //  printf("ok");
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


SEXP odp_make_chain_tm(SEXP tran, SEXP meas, SEXP info){
  
  
  Check_AnyTransformation_Ptr(tran); 
  Check_AnyMeasurement_Ptr(meas); 
  
  PROTECT(meas);
  PROTECT(tran);
  
  AnyTransformation *tran_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran);
  AnyMeasurement *meas_ptr = (AnyMeasurement *) R_ExternalPtrAddr(meas);
  
  FfiResult_____AnyMeasurement result = opendp_combinators__make_chain_pm(tran_ptr,meas_ptr);
  
  UNPROTECT(2);
  
  if(result.tag == Ok_____AnyMeasurement){
    
    //  printf("ok");
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)result.ok, AnyMeasurement_tag, info)); 
    R_RegisterCFinalizerEx(XPtr, odp_AnyMeasurement_finalizer,TRUE); 
    UNPROTECT(1);

    SEXP res = PROTECT(allocVector(VECSXP, 2));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, XPtr);

    SEXP names = PROTECT(allocVector(STRSXP, 2));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 2, mkChar("MeasurementPtr"));
    setAttrib(res, R_NamesSymbol, names);
    
    UNPROTECT(2);
    return(res);
  }
  
  // printf("no");
  
  return(extract_error(result.err));
  
}



SEXP odp_make_chain_tt(SEXP tran1, SEXP tran2, SEXP info){
  
  
  Check_AnyTransformation_Ptr(tran1); 
  Check_AnyTransformation_Ptr(tran2); 

  PROTECT(tran1);
  PROTECT(tran2);
  
  AnyTransformation *tran1_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran1);
  AnyTransformation *tran2_ptr = (AnyTransformation *) R_ExternalPtrAddr(tran2);
  
  FfiResult_____AnyTransformation result = opendp_combinators__make_chain_tt(tran1_ptr,tran2_ptr);
  
  UNPROTECT(2);
  
  if(result.tag == Ok_____AnyTransformation){
    
    //  printf("ok");
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
  
  // printf("no");
  
  return(extract_error(result.err));
  
}

