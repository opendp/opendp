#include <Rinternals.h>
#include "opendp.h"

// /**
//  * A Transformation with all generic types filled by Any types. This is the type of Transformation
//  * passed back and forth over FFI.
//  */
// typedef struct AnyTransformation AnyTransformation;

// /**
//  * A Measurement with all generic types filled by Any types. This is the type of Measurements
//  * passed back and forth over FFI.
//  */
// typedef struct AnyMeasurement AnyMeasurement;

typedef void c_void;
typedef char c_char;
typedef unsigned char c_bool;
typedef struct RetainedSexp RetainedSexp;

char *rt_to_string(SEXP type_name);
SEXP get_private_func(const char *func_name);
SEXP parse_runtime_type(const char *type_name);

AnyObject *sexp_to_anyobjectptr(SEXP data, SEXP type_name);
SEXP anyobjectptr_to_sexp(AnyObject *obj);
ExtrinsicObject *sexp_to_extrinsicobjectptr(SEXP value);
CallbackFn sexp_to_callbackfn(SEXP function, const char *type_name);
TransitionFn sexp_to_transitionfn(SEXP function, const char *type_name);
void callbackfn_release(CallbackFn *function);
void transitionfn_release(TransitionFn *transition);

void *sexp_to_voidptr(SEXP input, SEXP rust_type);
SEXP voidptr_to_sexp(void *input, SEXP rust_type, size_t len);

SEXP anymeasurement_to_sexp(AnyMeasurement *input);
const char *sexp_to_charptr(SEXP type_name);

SEXP extract_error(FfiError *err);

unsigned char str_equal(const char *str1, const char *str2);

void init_udf_support(void);
RetainedSexp *new_retained_sexp(SEXP value);
SEXP extrinsic_object_to_sexp(const ExtrinsicObject *input);
