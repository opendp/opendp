/* this file contains the AnyObject related calls */


#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>
#include <R_ext/Callbacks.h>
// #include "opendp_base.h"
// #include "opendp.h"

// Import C headers for rust API
#include "Ropendp.h"



int is_measVec(SEXP x) {
  return Rf_inherits(x, "measVec") ? 1 : 0;
}

SEXP extract_error(struct FfiError *err){
  int l1 = strlen(err->variant);
  int l2 = strlen(err->message);
  char *msg = (char *)R_alloc(1, (l1+l2+6) * sizeof(char));
  sprintf(msg, "[%s] : %s",err->variant,err->message);
  
  error(msg);
  return(R_NilValue);
}  
// finalizers
static void odp_AnyObject_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_AnyObject_Ptr(XPtr); 
 // Rprintf("\n*** finalizing AnyObject ***\n\n");
  struct AnyObject *ptr = (struct AnyObject *) R_ExternalPtrAddr(XPtr);
  opendp_data__object_free(ptr);
  
  R_ClearExternalPtr(XPtr);
}


static void odp_SMDCurve_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_SMDCurve_Ptr(XPtr); 
   //Rprintf("\n*** finalizing SMDCurve ***\n\n");
  struct AnyObject *ptr = (struct AnyObject *) R_ExternalPtrAddr(XPtr);
  opendp_data__object_free(ptr);
  
  R_ClearExternalPtr(XPtr);
}




// finalizers
static void double_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_double_Ptr(XPtr); 
//  Rprintf("\n*** finalizing double ***\n\n");
  double *ptr = (double *) R_ExternalPtrAddr(XPtr);
  free(ptr);
  
  R_ClearExternalPtr(XPtr);
}

static void int_finalizer(SEXP XPtr){
  Check_int_Ptr(XPtr); 
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  //Check_AnyObject_Ptr(XPtr); 
////  Rprintf("\n*** finalizing int ***\n\n");
  int *ptr = (int *) R_ExternalPtrAddr(XPtr);
  free(ptr);
  
  R_ClearExternalPtr(XPtr);
}


static void char_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_char_Ptr(XPtr); 
  
  //Check_AnyObject_Ptr(XPtr); 
 //  Rprintf("\n*** finalizing char ***\n\n");
  char *ptr = (char *) R_ExternalPtrAddr(XPtr);
  free(ptr);
  
  R_ClearExternalPtr(XPtr);
}

static void charVec_finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  Check_charVec_Ptr(XPtr); 
  
  //Check_AnyObject_Ptr(XPtr); 
//  Rprintf("\n*** finalizing charVec ***\n\n");
  char **ptr = (char **) R_ExternalPtrAddr(XPtr);
  int len = sizeof(ptr) / sizeof(ptr[0]);
  int i;
  for (i=0;i<len;i++)
    free(ptr[i]);
  free(ptr);
  
  R_ClearExternalPtr(XPtr);
}

struct AnyObject *SEXP2AnyObjectWithType(SEXP data, char *destType) {
  char *typename = Calloc(9, char);
  int len;
  char **str; 
  void *val;
  void *val1, *val2;
  int i;
  void **array;
  SEXP retkeys;
  SEXP retvals;
  
  PROTECT(data);
  printf("\nSEXP2AnyObjectWithType: start, TYPEOF = %d, destType=%s\n", TYPEOF(data), destType);
  
  if(TYPEOF(data) != ENVSXP){
    len = LENGTH(data);
    str = malloc(sizeof(char *)*(len));
  }
  switch( TYPEOF(data) ) {
  case INTSXP:
    // printf("\nAn integer ");
    if( (len > 1) | (strcmp(destType,"Vec<u32>")==0) | (strcmp(destType,"Vec<i32>")==0) ) {
      if(strcmp(destType,"Vec<u32>")==0){
        strcpy(typename, "Vec<u32>");
      } else {
        strcpy(typename, "Vec<i32>");
      }
      // printf("vector\n");
    } else {
      if(strcmp(destType,"u32")==0){
        strcpy(typename, "u32");
      } else {
        strcpy(typename, "i32");
      }
    }
    if(strcmp(destType,"u32")==0){
      val = (unsigned int*)( INTEGER(data) );
      // unsigned int unsigned_x = (unsigned int) INTEGER(data)[0];
      // val = *unsigned_x;
    } else {
      val = INTEGER(data);
    }
    break;
  case VECSXP:
  case LISTSXP:  // these are tuples
    // printf("\nTYPEOF = %d, len=%d, is measVec=%d\n",TYPEOF(data),len,  is_measVec(data) );
    array= (void **)malloc(len*sizeof(void *));
    char *objtype= Calloc(20, char); // not sure which is the max
    int obj_len=0;
    
    if(is_measVec(data)){ // is a vector of measurement external pointers
      for(i=0;i<len;i++){
        array[i] = (void *)(struct AnyObject *) R_ExternalPtrAddr(VECTOR_ELT(data,i));
      }
      strcpy(objtype,"Vec<AnyMeasurementPtr>");
      obj_len= len;
    } else { // is a tuple
      for(i=0;i<len;i++){
        array[i] = (void *)R2C(VECTOR_ELT(data,i));
      }
      if(TYPEOF(VECTOR_ELT(data,0)) == INTSXP)
        strcpy(objtype,"(i32, i32)");
      
      if(TYPEOF(VECTOR_ELT(data,0)) == REALSXP)
        strcpy(objtype,"(f64, f64)");
      obj_len = 2;
    }
    
    FfiSlice slice = {array, obj_len};
    
    FfiResult_____AnyObject tuple = opendp_data__slice_as_object(&slice, objtype);
    
    UNPROTECT(1);
    
    if(tuple.tag == Ok_____AnyObject){
      return(tuple.ok);
    }
    extract_error(tuple.err);
    break;
  case REALSXP:
    if( (len > 1) | (strcmp(destType,"Vec<f64>")==0) ) {
      strcpy(typename, "Vec<f64>");
    } else {
      strcpy(typename, "f64");
    }
    val = REAL(data);
    break;
  case STRSXP:
    for (i = 0;i < len; i++) {
      str[i] = malloc(sizeof(char)*length(STRING_ELT(data,i)));
      strcpy(str[i], CHAR(STRING_ELT(data,i)) );
    }
    if(len>1){
      strcpy(typename, "Vec<String>");
      val = str;
    } else {
      strcpy(typename, "String");
      val = *str;
    } 
    break;  
  case ENVSXP:
    retkeys = PROTECT(findVarInFrame(data, install("keys")));
    retvals = PROTECT(findVarInFrame(data, install("values")));
    val1 = SEXP2AnyObject(retkeys);
    val2 = SEXP2AnyObject(retvals);
    void *hash[2];
    hash[0] = (struct AnyObject *)val1;
    hash[1] = (struct AnyObject *)val2;
    len = 2;
    strcpy(typename,"HashMap<String, f64>");
    FfiSlice tupslice = { hash, len};
    FfiResult_____AnyObject result2 = opendp_data__slice_as_object(&tupslice, typename);
    UNPROTECT(3);
    //  printf("\nResult tag (HashMap)= %d\n",result2.tag);
    if(result2.tag == Ok_____AnyObject){
      return(result2.ok);
    }
    extract_error(result2.err);
    break;
  }
  
  //  printf("\n here: typename=%s\n",typename);
  if(TYPEOF(data)==ENVSXP){
    printf("\n ENV, not good\n");
    return(NULL);
  }
  
  FfiSlice slice = {val, len};
  FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);
  
  // printf("\nResult tag = %d\n",result.tag);
  if(TYPEOF(data)==STRSXP){
    //   printf("\n freeing string, len=%d\n",len);
    for (i=0;i<len;i++){
      free(str[i]);
    }
  }
  free(str);
  Free(typename);  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyObject){
    return(result.ok);  
  }
  
  extract_error(result.err);
  return(NULL); // this is needed only to avoid compilation warnings
}



// converts an R object to an AnyObject
struct AnyObject *SEXP2AnyObject(SEXP data) {
  char *typename = Calloc(9, char);
  int len;
  char **str; 
  void *val;
  void *val1, *val2;
  int i;
  void **array;
  SEXP retkeys;
  SEXP retvals;
  
  PROTECT(data);
 //printf("\nSEXP2AnyObject: start, TYPEOF = %d\n", TYPEOF(data));
  
   if(TYPEOF(data) != ENVSXP){
    len = LENGTH(data);
    str = malloc(sizeof(char *)*(len));
   }
  switch( TYPEOF(data) ) {
  case INTSXP:
   // printf("\nAn integer ");
    if (len != 1) {
      strcpy(typename, "Vec<i32>");
     // printf("vector\n");
    } else {
      strcpy(typename, "i32");
  //    printf("\n");
    }
    val = INTEGER(data);
  break;
  case VECSXP:
  case LISTSXP:  // these are tuples
   // printf("\nTYPEOF = %d, len=%d, is measVec=%d\n",TYPEOF(data),len,  is_measVec(data) );
    array= (void **)malloc(len*sizeof(void *));
    char *objtype= Calloc(20, char); // not sure which is the max
    int obj_len=0;
    
    if(is_measVec(data)){ // is a vector of measurement external pointers
      for(i=0;i<len;i++){
        array[i] = (void *)(struct AnyObject *) R_ExternalPtrAddr(VECTOR_ELT(data,i));
      }
      strcpy(objtype,"Vec<AnyMeasurementPtr>");
      obj_len= len;
    } else { // is a tuple
      for(i=0;i<len;i++){
        array[i] = (void *)R2C(VECTOR_ELT(data,i));
      }
       if(TYPEOF(VECTOR_ELT(data,0)) == INTSXP)
        strcpy(objtype,"(i32, i32)");
      
      if(TYPEOF(VECTOR_ELT(data,0)) == REALSXP)
        strcpy(objtype,"(f64, f64)");
      obj_len = 2;
    }
   
    FfiSlice slice = {array, obj_len};
    
    FfiResult_____AnyObject tuple = opendp_data__slice_as_object(&slice, objtype);
    
    UNPROTECT(1);
    
    if(tuple.tag == Ok_____AnyObject){
      return(tuple.ok);
    }
    extract_error(tuple.err);
  break;
  case REALSXP:
    if (len != 1) {
      strcpy(typename, "Vec<f64>");
    } else {
      strcpy(typename, "f64");
    }
    val = REAL(data);
  break;
  case STRSXP:
    for (i = 0;i < len; i++) {
      str[i] = malloc(sizeof(char)*length(STRING_ELT(data,i)));
      strcpy(str[i], CHAR(STRING_ELT(data,i)) );
    }
    if(len>1){
      strcpy(typename, "Vec<String>");
      val = str;
    } else {
      strcpy(typename, "String");
      val = *str;
    } 
  break;  
  case ENVSXP:
    retkeys = PROTECT(findVarInFrame(data, install("keys")));
    retvals = PROTECT(findVarInFrame(data, install("values")));
    val1 = SEXP2AnyObject(retkeys);
    val2 = SEXP2AnyObject(retvals);
    void *hash[2];
    hash[0] = (struct AnyObject *)val1;
    hash[1] = (struct AnyObject *)val2;
    len = 2;
    strcpy(typename,"HashMap<String, f64>");
    FfiSlice tupslice = { hash, len};
    FfiResult_____AnyObject result2 = opendp_data__slice_as_object(&tupslice, typename);
    UNPROTECT(3);
  //  printf("\nResult tag (HashMap)= %d\n",result2.tag);
    if(result2.tag == Ok_____AnyObject){
      return(result2.ok);
    }
    extract_error(result2.err);
    break;
  }
  
//  printf("\n here: typename=%s\n",typename);
  if(TYPEOF(data)==ENVSXP){
    printf("\n ENV, not good\n");
    return(NULL);
  }
  
  FfiSlice slice = {val, len};
  FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);
  
 // printf("\nResult tag = %d\n",result.tag);
  if(TYPEOF(data)==STRSXP){
 //   printf("\n freeing string, len=%d\n",len);
    for (i=0;i<len;i++){
       free(str[i]);
    }
  }
  free(str);
  Free(typename);  
  UNPROTECT(1);
  
  if(result.tag == Ok_____AnyObject){
    return(result.ok);  
  }
  
  extract_error(result.err);
  return(NULL); // this is needed only to avoid compilation warnings
}




// returns a void* to the underlying data, and saves the typename
// we need two versions, one for vectors and one for scalars
void *extract_pointer(SEXP val, char *typename, char **str) {
  int len = LENGTH(val);
  
  switch( TYPEOF(val) ) {
  case INTSXP:
    if (len != 1) {
      strcpy(typename, "Vec<i32>");
    } else {
      strcpy(typename, "i32");
    }
    //    printf("INTSXP\n");
    return INTEGER(val);
  case REALSXP:
    if (len != 1) {
      strcpy(typename, "Vec<f64>");
    } else {
      strcpy(typename, "f64");
    }
    //  printf("REALSXP\n");
    return REAL(val);
  case STRSXP:
    if (len != 1) {
      strcpy(typename, "Vec<String>");
  //    printf("\nSTRSXP:%s",typename);
    } else {
      strcpy(typename, "String");
//      printf("\nSTRSXP:%s",typename);
    }
    int i;
    for (i = 0;i < len; i++) {
      str[i] = malloc(sizeof(char)*length(STRING_ELT(val,i)));
      strcpy(str[i], CHAR(STRING_ELT(val,i)) );
    }
    if(len>1){
      return(str);
    } else {
      return(*str);
    } 
  default:
    return NULL;
  }
}


// embeds R objects to C pointers
void *R2C(SEXP data) {
  
  // This choice of size is the longest currently-supported typename.
 // int len = LENGTH(data);
  
  PROTECT(data);
  if(TYPEOF(data)==REALSXP){
    UNPROTECT(1);
    return(REAL(data));
  }
  
  if(TYPEOF(data)==INTSXP){
    UNPROTECT(1);
    return(INTEGER(data));
  }
  UNPROTECT(1);
  return(NULL);
}

SEXP odp_R_to_C(SEXP value, SEXP c_type, SEXP type_name) {
  
  // This choice of size is the longest currently-supported typename.
  int len = LENGTH(value);
  SEXP XPtr;
  
  PROTECT(c_type);
  PROTECT(type_name);
  PROTECT(value);
  
  if(TYPEOF(value) == VECSXP){ // we are creating a TUPLE
    // printf("\na list\n");
    // printf("\ntypeof value:%d\n", TYPEOF(value));
    SEXP retval = PROTECT(allocVector(VECSXP, len));
    SEXP retnames = PROTECT(allocVector(STRSXP, len));
    
    
    
    int i;
    for(i=0;i<len;i++){
      SEXP val = VECTOR_ELT(value,i);
      // printf("\n%d\n",i);
      // 
      //   printf("\nnumeric\n");
      //   printf("\nTYPEOF val:%d\n", TYPEOF(val));
      //   printf("\nlen c_type:%d, len type_name %d\n", length(STRING_ELT(c_type,i)), length(STRING_ELT(type_name,i)));
      //   printf("\nc_type:%s, type_name %s\n", CHAR(STRING_ELT(c_type,i)), CHAR(STRING_ELT(type_name,i)));
        SEXP tmpval = odp_R_to_C(val,mkString(CHAR(STRING_ELT(c_type,i))), mkString(CHAR(STRING_ELT(type_name,i))));
        SET_VECTOR_ELT(retval, i, tmpval);
        SET_STRING_ELT(retnames, i, mkChar(CHAR(STRING_ELT(type_name,i))));
    }
    UNPROTECT(5);
    setAttrib(retval, R_NamesSymbol, retnames);
    return(retval);
  }
  // printf("\nnot a list: typeof c_type=%d, type_name=%d\n",TYPEOF(c_type),TYPEOF(type_name));
  // printf("c_type len=%d, type_name len=%d",length(c_type),length(type_name));

    
 // const char *my_c_type = R_CHAR(STRING_ELT(c_type, 0));
  const char *my_type_name = R_CHAR(STRING_ELT(type_name, 0));
  
  
  // printf("\nc_type=%s, type_name=%s\n",my_c_type, my_type_name);
  if(strcmp(my_type_name,"Vec<String>")==0 | strcmp(my_type_name,"String")==0){
    // printf("\ntype_name: %s, len=%d", my_type_name, len);
    int i;
    char **str = malloc(sizeof(char *)*(len));
    for (i = 0;i < len; i++) {
      str[i] = malloc(sizeof(char)*length(STRING_ELT(value,i)));
      strcpy(str[i], CHAR(STRING_ELT(value,i)) );
    }
    
    if(len>1){
      XPtr = PROTECT(R_MakeExternalPtr(str, charVec_tag, R_NilValue));
      R_RegisterCFinalizerEx(XPtr, charVec_finalizer, TRUE); // important to register the proper finalizer
    } else {
      XPtr = PROTECT(R_MakeExternalPtr(*str, char_tag, R_NilValue));
      R_RegisterCFinalizerEx(XPtr, char_finalizer, TRUE); // important to register the proper finalizer
    } 
    
    UNPROTECT(4);
    return(XPtr);
  } // Vec<String> | String              
  

  if(strcmp(my_type_name,"Vec<i32>")==0 | strcmp(my_type_name,"i32")==0){
  //  printf("\ntype_name: %s, len=%d", my_type_name, len);
    int i;
    int *num = malloc(sizeof(int *)*len);
    for(i=0;i<len;i++)
      num[i] = INTEGER(value)[i];
    
    XPtr = PROTECT(R_MakeExternalPtr(num, int_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, int_finalizer, TRUE); // important to register the proper finalizer
    
    UNPROTECT(4);
    return(XPtr);
  } // Vec<i32> | i32        
  
  if(strcmp(my_type_name,"Vec<f64>")==0 | strcmp(my_type_name,"f64")==0){
  //  printf("\ntype_name: %s, len=%d", my_type_name, len);
    int i;
    double *num = malloc(sizeof(double *)*len);
    for(i=0;i<len;i++)
      num[i] = REAL(value)[i];
    
    XPtr = PROTECT(R_MakeExternalPtr(num, double_tag, R_NilValue));
    R_RegisterCFinalizerEx(XPtr, double_finalizer, TRUE); // important to register the proper finalizer

    UNPROTECT(4);
    return(XPtr);
  } // Vec<f64> | f64                        
  
  UNPROTECT(3);
  return(R_NilValue);
}


SEXP odp_slice_as_object(SEXP data) {
  
  // This choice of size is the longest currently-supported typename.
  char *typename = Calloc(9, char);
  int len = LENGTH(data);
  char **str = malloc(sizeof(char *)*(len));
  
  PROTECT(data);
  FfiSlice slice = { extract_pointer(data, typename, str), len};
  FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);
 
  int i;
  if(TYPEOF(data)==STRSXP){
    for (i=0;i<len;i++)
      free(str[i]);
  }
  free(str);
  
  
 //  printf("\ntypename=%s",typename);
  // convert the FfiSlice to an AnyObject (or error).
  // An AnyObject is an opaque C struct that contains the data in a rust-specific representation
  // AnyObjects may be used interchangeably in the majority of library APIs
  UNPROTECT(1);
  
  // TODO: result unwrapping and return opaque AnyObject struct
//  printf("Success or error: %d, typename = %s\n", result.tag, typename);
  //FfiResult_____c_char tname = opendp_data__object_type(result.ok);
//  printf("\ntname=%s",tname.ok);
  // 
  // the function returns the input just so that the code compiles
  //return data;
  if(result.tag == Ok_____AnyMeasurement){
    
//    SEXP XPtr = PROTECT(R_MakeExternalPtr(result.ok, R_NilValue, data));
    SEXP XPtr = PROTECT(R_MakeExternalPtr(result.ok, AnyObject_tag, data));
    R_RegisterCFinalizerEx(XPtr, odp_AnyObject_finalizer, TRUE); // important to register the proper finalizer
    UNPROTECT(1);
  
    SEXP res = PROTECT(allocVector(VECSXP, 4));
    SET_VECTOR_ELT(res, 0, ScalarInteger(result.tag));
    SET_VECTOR_ELT(res, 1, mkString(typename));
    SET_VECTOR_ELT(res, 2, ScalarInteger(len));
    SET_VECTOR_ELT(res, 3, XPtr);

  
    SEXP names = PROTECT(allocVector(STRSXP, 4));
    SET_STRING_ELT(names, 0, mkChar("error"));
    SET_STRING_ELT(names, 1, mkChar("type"));
    SET_STRING_ELT(names, 2, mkChar("length"));
    SET_STRING_ELT(names, 3, mkChar("AnyObjectPtr"));
    setAttrib(res, R_NamesSymbol, names);

    UNPROTECT(2);
  
    Free(typename);  
  
    return(res);
  }
  
  int l1 = strlen((result.err)->variant);
  int l2 = strlen((result.err)->message);
  char *msg = (char *)R_alloc(1, (l1+l2+6) * sizeof(char));
  sprintf(msg, "[%s] : %s",(result.err)->variant,(result.err)->message);

  error(msg);
  Free(typename);  
  return(R_NilValue);
}


SEXP odp_object_as_slice(SEXP obj){
  Check_AnyObject_Ptr(obj); 
  AnyObject *obj_ptr = (AnyObject *) R_ExternalPtrAddr(obj);
  
  return(int_object_as_slice(obj_ptr));
}

 
// this is an internal to C function, there will be a regular function
// that can be used from R
SEXP int_object_as_slice(struct AnyObject *obj){
 //printf("\ninside int_object_as_slice...");  
  FfiResult_____c_char typename1 = opendp_data__object_type(obj);
  //printf("typename (int_object_as_slice): %s\n", typename1.ok);
  if(strcmp(typename1.ok,"SMDCurve<f64>")==0){
    // this is a curve, we need to return as is
    SEXP XPtr = PROTECT(R_MakeExternalPtr((void *)obj, SMDCurve_tag, R_NilValue)); 
    R_RegisterCFinalizerEx(XPtr, odp_SMDCurve_finalizer,TRUE); 
    UNPROTECT(1);
    
    SEXP res = PROTECT(allocVector(VECSXP, 1));
    SET_VECTOR_ELT(res, 0, XPtr);
    
    SEXP names = PROTECT(allocVector(STRSXP, 1));
    SET_STRING_ELT(names, 0, mkChar("SMDCurvePtr"));
    setAttrib(res, R_NamesSymbol, names);
    SEXP class_SMD = PROTECT(mkString("SMDCurve"));
    setAttrib(res, R_ClassSymbol, class_SMD);
    UNPROTECT(3);
    return(res);
  }
  
  FfiResult_____FfiSlice val = opendp_data__object_as_slice(obj);
  //printf("Success or error (object as slice): %d\n", val.tag);
  
  if(val.tag !=  Ok_____FfiSlice)
    return(extract_error(val.err));
  
  int len = (int)val.ok->len;
  
  FfiResult_____c_char typename = opendp_data__object_type(obj);
  //printf("typename (object as slice): %s\n", typename.ok);
  
  if(val.tag !=  Ok_____FfiSlice)
    return(extract_error(val.err));
  
  //printf("strcmp:%d",strcmp(typename.ok,"i32"));
  if(strcmp(typename.ok,"Vec<AnyObject>")==0){
    FfiResult_____FfiSlice valptr = opendp_data__ffislice_of_anyobjectptrs(val.ok);
   // printf("Success or error (opendp_data__ffislice_of_anyobjectptrs): %d\n", valptr.tag);
    if(valptr.tag !=  Ok_____FfiSlice)
      return(extract_error(valptr.err));
    int lenptr = (int)valptr.ok->len;
    
    SEXP result = PROTECT(allocVector(VECSXP, lenptr));
    int i;
    void **ptr = (void **)(valptr.ok->ptr);
    for(i=0; i<lenptr;i++){
      SET_VECTOR_ELT(result, i, int_object_as_slice(ptr[i]));
    }
    UNPROTECT(1);
    return(result);   
  }
  
  if(strcmp(typename.ok,"(f64, f64)")==0){
    int i;
    double **vals = (double **)val.ok->ptr;
    int lenptr = (int)val.ok->len;
    
    SEXP result = PROTECT(allocVector(REALSXP, lenptr));
    for(i=0; i<lenptr;i++){
      REAL(result)[i] = *vals[i];
    }
    UNPROTECT(1);
    return(result);
  }
  if((strcmp(typename.ok,"Vec<f64>")==0)){
    // printf("dp-meas = %d", *(int *)val.ok->ptr);
//    printf("len=%d",len);
    int i;
    SEXP result = PROTECT(allocVector(REALSXP, len));
    double *vals = (double *)val.ok->ptr;
    for(i=0; i<len;i++){
      REAL(result)[i] = vals[i];
    }
    UNPROTECT(1);
    return(result);    
  }

    if(strcmp(typename.ok,"f64")==0){
     //printf("dp-meas = %f", *(double *)val.ok->ptr);
    return(ScalarReal(*(double *)(val.ok->ptr)));
  }
  
  if(strcmp(typename.ok,"i32")==0){
    // printf("dp-meas = %d", *(int *)val.ok->ptr);
    return(ScalarInteger(*(int *)(val.ok->ptr)));
  }
  
  if(strcmp(typename.ok,"u32")==0){
    // printf("dp-meas = %d", *(int *)val.ok->ptr);
    unsigned int newdata = (unsigned int)( *(int *)(val.ok->ptr) );
    
    return(ScalarInteger(newdata));
  }
  if(strcmp(typename.ok,"Vec<i32>")==0){
    // printf("dp-meas = %d", *(int *)val.ok->ptr);
    //    printf("len=%d",len);
    int i;
    SEXP result = PROTECT(allocVector(INTSXP, len));
    int *vals = (int *)val.ok->ptr;
    for(i=0; i<len;i++){
      INTEGER(result)[i] = vals[i];
    }
    UNPROTECT(1);
    return(result);    
  }
  if(strcmp(typename.ok,"String")==0){
    //    printf("dp-meas = %s\n", (char *)val.ok->ptr);
    return(ScalarString(mkChar((char *)(val.ok->ptr))));
  }
  
  if(strcmp(typename.ok,"Vec<String>")==0){
    // printf("dp-meas = %d", *(int *)val.ok->ptr);
//    printf("len=%d",len);
    int i;
    SEXP result = PROTECT(allocVector(STRSXP, len));
    char **vals = (char **)val.ok->ptr;
    for(i=0; i<len;i++){
      SET_STRING_ELT(result, i, mkChar(vals[i]));
    }
    UNPROTECT(1);
    return(result);    
  }
  
  if(strcmp(typename.ok,"HashMap<String, f64>")==0){
    
   // printf("\nneed to upack an HashMap\n");
    int errorOccurred;
    void **vals = (void **)val.ok->ptr;
    SEXP keys = int_object_as_slice(vals[0]);
    SEXP values = int_object_as_slice(vals[1]);
    SEXP R_fcall = lang3(install("hashmap"), keys, values);
    SEXP ret = R_tryEval(R_fcall, R_GlobalEnv, &errorOccurred);
   // printf("R returned: %d", errorOccurred);
    if (!errorOccurred){
      return(ret);
    } else {
      return(R_NilValue);
    }
  }
  
  return(R_NilValue);
}


SEXP processlist(SEXP lst){
  int i,elLength;
  int len = length(lst);
  char **target = malloc(sizeof(char *)*len);
  for (i = 0;i < len; i++) {
    elLength = length(STRING_ELT(lst,i));
    printf("\nlen = %d - %s",elLength, CHAR(STRING_ELT(lst,i)));
    target[i] = malloc(sizeof(char ) * elLength);
    strcpy(target[i], CHAR(STRING_ELT(lst,i)) );
  }
  
  for (i = 0;i < len; i++) {
    printf("target[%d]: %s\n",i, target[i]);
  }
  
  for (i=0;i<len;i++)
    free(target[i]);
  free(target);
  return R_NilValue;
}




 
