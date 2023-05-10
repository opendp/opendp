#include <R.h>
#include <Rmath.h>
#include <R_ext/Boolean.h>
#include <R_ext/Rdynload.h>
#include <Rdefines.h>
#include <Rinternals.h>
#include <R_ext/Complex.h>


#include "Ropendp.h"

  
// taken from here with some changes
// https://stackoverflow.com/questions/7032617/storing-c-objects-in-r
// See also: http://homepage.divms.uiowa.edu/~luke/R/simpleref.html
// My old pkg RRP  https://github.com/siacus/rrp
// and of course R Devel guide
// https://cran.rstudio.com/doc/manuals/r-devel/R-exts.html#External-pointers-and-weak-references


// finalizer is call when the object is clear on the R side with
// something like obj <- NULL

static void _finalizer(SEXP XPtr){
  if (NULL == R_ExternalPtrAddr(XPtr))
    return;
  CHECK_OPENDP_OBJECT(XPtr); 
  Rprintf("finalizing\n");
  char *ptr = (char *) R_ExternalPtrAddr(XPtr);
  Free(ptr);
  R_ClearExternalPtr(XPtr);
}

const int N_MAX=15;

// this creates a C object and embeds it in an R object
SEXP create(SEXP info){
  char *x = Calloc(N_MAX, char);
  snprintf(x, N_MAX, "I am an OpenDP obj");
  SEXP XPtr = PROTECT(R_MakeExternalPtr(x, OPENDP_tag, info));
  R_RegisterCFinalizerEx(XPtr, _finalizer, TRUE); // important to register the proper fnalizer
  UNPROTECT(1);
  
  SEXP res = PROTECT(allocVector(VECSXP, 3));
  SET_VECTOR_ELT(res, 0, XPtr);
  SET_VECTOR_ELT(res, 1, mkString(x));
  SET_VECTOR_ELT(res, 2, mkString("odp obj"));
  
  SEXP names = PROTECT(allocVector(STRSXP, 3));
  SET_STRING_ELT(names, 0, mkChar("Ptr"));
  SET_STRING_ELT(names, 1, mkChar("value"));
  SET_STRING_ELT(names, 2, mkChar("type"));
  setAttrib(res, R_NamesSymbol, names);
  //  printf("\n5");
  UNPROTECT(2);
  return (res);
}

SEXP get(SEXP XPtr){
  return mkString((char *) R_ExternalPtrAddr(XPtr));
}


SEXP set(SEXP XPtr, SEXP str){
  char *x = (char *) R_ExternalPtrAddr(XPtr);
  snprintf(x, N_MAX, "%s", R_CHAR(STRING_ELT(str, 0)));
  return ScalarLogical(TRUE);
}


/* function bodies */
SEXP apply_Fun(SEXP x, SEXP f, SEXP rho)
{
  SEXP ans;
  double *retval, *myx;
  if(!isNumeric(x)) error("`x' must be numeric");
  PROTECT(x = AS_NUMERIC(x));
  PROTECT(ans = NEW_NUMERIC(1));
  retval = NUMERIC_POINTER(ans);
  myx = NUMERIC_POINTER(x); /* maps myx double pointer to a R numeric pointer */

  *retval =  feval(*myx, f, rho);   
  UNPROTECT(2);
  return( ans ); 
}


SEXP square_It(SEXP x)
{
  double myx=0, *retval;
  SEXP ans;
  
  if(!isNumeric(x)) error("`x' must be numeric"); /* sanity check */
  PROTECT(x = AS_NUMERIC(x)); /* protects the input x */
  PROTECT(ans = NEW_NUMERIC(1)); /* creates and protects the return value */
  
  myx = *NUMERIC_POINTER(x); /* copies 'x' numericn into 'myx' */

  retval = NUMERIC_POINTER(ans); /* maps 'retval' double pointer to a the 'ans' R numeric pointer */
  /* here I could have done similarly to the above, just to show the pointer version */
  
  *retval = myx*myx; /* do the squaring */
  UNPROTECT(2); /* unprotect objects before returning to R */
  return(ans); 
}




SEXP mkans(double x)
{
    SEXP ans;
    PROTECT(ans = allocVector(REALSXP, 1));
    REAL(ans)[0] = x;
    UNPROTECT(1);
    return ans;
}

double feval(double x, SEXP f, SEXP rho)
{
    double val;
    SEXP R_fcall;    
    SEXP sx = install("x");
    defineVar(sx, mkans(x), rho);
    PROTECT(R_fcall = lang2(f, mkans(x)));
    val = *NUMERIC_POINTER(eval(R_fcall, rho));
    UNPROTECT(1);
    return(val);
}


 
