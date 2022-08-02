#define R_NO_REMAP
#define STRICT_R_HEADERS

#include <R.h>
#include <Rinternals.h>
#include <stdio.h>

// Import C headers for rust API
#include "opendp_base.h"
#include "opendp.h"

// returns a void* to the underlying data, and saves the typename
void *extract_pointer(SEXP val, char *typename) {

    switch( TYPEOF(val) ) {
        case INTSXP:
            strcpy(typename, "Vec<i32>\0");
            printf("INTSXP\n");
            return INTEGER(val);
        case REALSXP:
            strcpy(typename, "Vec<f64>\0");
            printf("REALSXP\n");
            return REAL(val);
        default:
            return NULL;
    }
}

SEXP slice_as_object__wrapper(SEXP data) {

    // This choice of size is the longest currently-supported typename.
    char *typename = malloc(9);

    // construct an FfiSlice containing the data
    FfiSlice slice = { extract_pointer(data, typename), LENGTH(data)};

    // convert the FfiSlice to an AnyObject (or error).
    // An AnyObject is an opaque C struct that contains the data in a rust-specific representation
    // AnyObjects may be used interchangeably in the majority of library APIs
    FfiResult_____AnyObject result = opendp_data__slice_as_object(&slice, typename);

    // TODO: result unwrapping and return opaque AnyObject struct
    printf("Success or error: %d\n", result.tag);

    // the function returns the input just so that the code compiles
    return data;
}

// R boilerplate
static const R_CallMethodDef CallEntries[] = {
        // name of routine, pointer to function, number of arguments
        {"slice_as_object__wrapper", (DL_FUNC) &slice_as_object__wrapper, 1},
        // ...repeat for each function exported by opendp.so, used by the R library

        // why? This boilerplate line is everywhere
        {NULL, NULL, 0}
};

void R_init_opendp(DllInfo *dll) {
    R_registerRoutines(dll, NULL, CallEntries, NULL, NULL);
    R_useDynamicSymbols(dll, FALSE);
}